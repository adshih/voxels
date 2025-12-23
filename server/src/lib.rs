mod message;

use quinn::rustls::pki_types::PrivatePkcs8KeyDer;
use quinn::{Connection, Endpoint, ServerConfig};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;
use voxel_world::VoxelWorld;

pub use message::Message;
use voxel_world::commands::WorldCommand;
use voxel_world::events::WorldEvent;

const TICK_RATE: f32 = 60.0;
const DT: f32 = 1.0 / TICK_RATE;

struct ClientInfo {
    conn: Connection,
    name: String,
}

struct ServerState {
    world: VoxelWorld,
    clients: HashMap<u32, ClientInfo>,
}

pub struct Server {
    endpoint: Endpoint,
    state: Arc<RwLock<ServerState>>,
}

impl Server {
    pub async fn bind(addr: SocketAddr, config: ServerConfig) -> anyhow::Result<Self> {
        let endpoint = Endpoint::server(config, addr)?;
        println!("Listening on {}", addr);

        Ok(Self {
            endpoint,
            state: Arc::new(RwLock::new(ServerState {
                world: VoxelWorld::new(123),
                clients: HashMap::new(),
            })),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.endpoint.local_addr().unwrap()
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let state = self.state.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs_f32(DT));

            loop {
                interval.tick().await;

                let events = {
                    let mut state = state.write().await;
                    state.world.tick(DT)
                };

                for event in events {
                    handle_event(&state, event).await;
                }
            }
        });

        while let Some(conn) = self.endpoint.accept().await {
            let state = self.state.clone();
            tokio::spawn(async move {
                if let Ok(connection) = conn.await {
                    handle_player(connection, state).await;
                }
            });
        }

        Ok(())
    }
}

async fn handle_player(conn: Connection, state: Arc<RwLock<ServerState>>) {
    let mut client_id: Option<u32> = None;

    loop {
        tokio::select! {
            result = conn.read_datagram() => {
                let Ok(data) = result else { break };
                if let Ok(msg) = Message::deserialize(&data)
                    && let Some(id) = client_id
                {
                    handle_message(msg, id, &state).await;
                }
            }

            result = conn.accept_bi() => {
                let Ok((mut send, mut recv)) = result else { break };
                let mut buf = vec![0u8; 1024];

                if let Ok(Some(n)) = recv.read(&mut buf).await
                    && let Ok(msg) = Message::deserialize(&buf[..n])
                {
                    match &msg {
                        Message::Connect { name } => {
                           let id = register_player(&state).await;
                           client_id = Some(id);

                           send_connect_ack(id, &mut send).await;
                           send_existing_players(&conn, &state).await;

                           insert_client(id, conn.clone(), name, &state).await;
                           broadcast_player_joined(id, name, &state).await;
                        }
                        _ => {
                            if let Some(id) = client_id {
                                handle_message(msg, id, &state).await;
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(id) = client_id {
        remove_client(id, &state).await;
    }
}

async fn register_player(state: &Arc<RwLock<ServerState>>) -> u32 {
    let mut state = state.write().await;
    let id = state.world.add_player();
    println!("Player (id: {}) connected", id);
    id
}

async fn send_connect_ack(id: u32, send: &mut quinn::SendStream) {
    let msg = Message::ConnectAck { client_id: id };
    if let Ok(bytes) = msg.serialize() {
        let _ = send.write_all(&bytes).await;
        let _ = send.finish();
    }
}

async fn send_existing_players(conn: &Connection, state: &Arc<RwLock<ServerState>>) {
    let state = state.read().await;

    for (&id, client) in &state.clients {
        send_reliable(
            conn,
            &Message::PlayerJoined {
                client_id: id,
                name: client.name.clone(),
            },
        )
        .await;
    }
}

async fn broadcast_player_joined(id: u32, name: &str, state: &Arc<RwLock<ServerState>>) {
    let state = state.read().await;

    for (&other_id, client) in &state.clients {
        if other_id != id {
            send_reliable(
                &client.conn,
                &Message::PlayerJoined {
                    client_id: id,
                    name: name.to_string(),
                },
            )
            .await;
        }
    }
}

async fn insert_client(id: u32, conn: Connection, name: &str, state: &Arc<RwLock<ServerState>>) {
    let mut state = state.write().await;
    state.clients.insert(
        id,
        ClientInfo {
            conn,
            name: name.to_string(),
        },
    );
}

async fn handle_message(msg: Message, id: u32, state: &Arc<RwLock<ServerState>>) {
    #[allow(clippy::single_match)]
    match msg {
        Message::Input { input } => {
            let mut state = state.write().await;
            state.world.execute(WorldCommand::PlayerMove { id, input });
        }
        _ => {}
    }
}

async fn remove_client(id: u32, state: &Arc<RwLock<ServerState>>) {
    let mut state = state.write().await;
    let Some(client) = state.clients.remove(&id) else {
        return;
    };

    println!("{} (id: {}) disconnected", client.name, id);

    let msg = Message::PlayerLeft {
        client_id: id,
        name: client.name,
    };
    for client in state.clients.values() {
        send_reliable(&client.conn, &msg).await;
    }
}

async fn handle_event(state: &Arc<RwLock<ServerState>>, event: WorldEvent) {
    match event {
        WorldEvent::PlayerMoved { id, pos, look } => {
            let state = state.read().await;
            let msg = Message::PositionUpdate {
                client_id: id,
                pos,
                look,
            };

            let bytes = msg.serialize().unwrap();

            for client in state.clients.values() {
                let _ = client.conn.send_datagram(bytes.clone().into());
            }
        }
        WorldEvent::ChunkLoaded {
            for_player,
            pos,
            data,
        } => {
            let conn = {
                let state = state.read().await;
                state.clients.get(&for_player).map(|c| c.conn.clone())
            };

            if let Some(conn) = conn {
                tokio::spawn(async move {
                    let msg = Message::ChunkLoaded { pos, data };
                    send_reliable(&conn, &msg).await;
                });
            }
        }
    }
}

async fn send_reliable(conn: &Connection, msg: &Message) {
    if let Ok(mut send) = conn.open_uni().await
        && let Ok(bytes) = msg.serialize()
    {
        let _ = send.write_all(&bytes).await;
        let _ = send.finish();
    }
}

pub fn configure_server() -> anyhow::Result<ServerConfig> {
    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])?;
    let key = PrivatePkcs8KeyDer::from(cert.signing_key.serialize_der());
    Ok(ServerConfig::with_single_cert(
        vec![cert.cert.into()],
        key.into(),
    )?)
}
