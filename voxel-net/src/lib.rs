use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use quinn::{Connection as QuicConnection, Endpoint, ServerConfig};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{
    RwLock,
    mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel},
};
use voxel_world::{
    VoxelWorld,
    command::WorldCommand,
    envelope::Envelope,
    event::WorldEvent,
    request::{Call, Connect, PendingRequest, Ping, WorldRequest},
};

const MAX_MSG_SIZE: usize = 1024 * 1024; // 1mb

pub struct Server {
    world: VoxelWorld,
    endpoint: Endpoint,
    clients: Arc<RwLock<HashMap<u32, UnboundedSender<WorldEvent>>>>,
}

impl Server {
    pub async fn bind(addr: SocketAddr, config: ServerConfig) -> anyhow::Result<Self> {
        let endpoint = Endpoint::server(config, addr)?;
        println!("Listening on {}", addr);

        Ok(Self {
            endpoint,
            world: VoxelWorld::new(123),
            clients: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let Server {
            world,
            endpoint,
            clients,
        } = self;

        let (cmd_tx, cmd_rx) = unbounded_channel();
        let (req_tx, req_rx) = unbounded_channel();
        let (event_tx, event_rx) = unbounded_channel();

        std::thread::spawn(move || world.run(cmd_rx, req_rx, event_tx));

        tokio::select! {
            _ = dispatch(event_rx, clients.clone()) => (),
            _ = accept_connections(endpoint, cmd_tx, req_tx, clients) => ()
        }

        Ok(())
    }
}

async fn dispatch(
    mut event_rx: UnboundedReceiver<Envelope<WorldEvent>>,
    clients: Arc<RwLock<HashMap<u32, UnboundedSender<WorldEvent>>>>,
) {
    while let Some(envelope) = event_rx.recv().await {
        let clients = clients.read().await;

        match &envelope.to {
            None => {
                for tx in clients.values() {
                    let _ = tx.send(envelope.payload.clone());
                }
            }
            Some(id) => {
                if let Some(tx) = clients.get(id) {
                    let _ = tx.send(envelope.payload);
                }
            }
        }
    }
}

async fn accept_connections(
    endpoint: Endpoint,
    cmd_tx: UnboundedSender<Envelope<WorldCommand>>,
    req_tx: UnboundedSender<PendingRequest>,
    clients: Arc<RwLock<HashMap<u32, UnboundedSender<WorldEvent>>>>,
) {
    while let Some(incoming) = endpoint.accept().await {
        let cmd_tx = cmd_tx.clone();
        let req_tx = req_tx.clone();
        let clients = clients.clone();

        tokio::spawn(async move {
            if let Ok(connection) = incoming.await
                && let Err(e) = session(connection, cmd_tx, req_tx, clients).await
            {
                eprintln!("session error: {e}");
            }
        });
    }
}

async fn session(
    connection: QuicConnection,
    cmd_tx: UnboundedSender<Envelope<WorldCommand>>,
    req_tx: UnboundedSender<PendingRequest>,
    clients: Arc<RwLock<HashMap<u32, UnboundedSender<WorldEvent>>>>,
) -> anyhow::Result<()> {
    let (id, name) = handshake(&connection, &req_tx).await?;
    println!("{name}[{id}] joined");

    let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
    clients.write().await.insert(id, event_tx);

    tokio::select! {
        _ = receive_commands(connection.clone(), cmd_tx.clone(), id) => (),
        _ = handle_requests(connection.clone(), req_tx.clone()) => (),
        _ = send_events(connection.clone(), event_rx) => ()
    }

    cmd_tx.send(Envelope::from(id, WorldCommand::Disconnect))?;
    clients.write().await.remove(&id);
    connection.close(0u32.into(), b"goodbye");

    println!("{name}[{id}] left");

    Ok(())
}

async fn handshake(
    connection: &QuicConnection,
    req_tx: &UnboundedSender<PendingRequest>,
) -> anyhow::Result<(u32, String)> {
    let (mut send, mut recv) = connection.accept_bi().await?;
    let bytes = recv.read_to_end(MAX_MSG_SIZE).await?;
    let connect: Connect = deserialize(&bytes)?;

    let (call, rx) = Call::new(connect.clone());
    req_tx.send(PendingRequest::Connect(call))?;
    let id = rx.await?;

    send.write_all(&serialize(&id)).await?;
    send.finish()?;

    Ok((id, connect.name))
}

async fn receive_commands(
    connection: QuicConnection,
    cmd_tx: UnboundedSender<Envelope<WorldCommand>>,
    id: u32,
) -> anyhow::Result<()> {
    loop {
        let data = connection.read_datagram().await?;
        let cmd: WorldCommand = deserialize(&data)?;
        cmd_tx.send(Envelope::from(id, cmd))?;
    }
}

async fn handle_requests(
    connection: QuicConnection,
    req_tx: UnboundedSender<PendingRequest>,
) -> anyhow::Result<()> {
    loop {
        let (mut send, mut recv) = connection.accept_bi().await?;
        let bytes = recv.read_to_end(MAX_MSG_SIZE).await?;
        let req: WorldRequest = deserialize(&bytes)?;

        #[allow(clippy::single_match)]
        match req {
            WorldRequest::Ping => {
                let (call, rx) = Call::new(Ping);
                req_tx.send(PendingRequest::Ping(call))?;
                let pong = rx.await?;

                send.write_all(&serialize(&pong)).await?;
                send.finish()?;
            }
            _ => {}
        }
    }
}

async fn send_events(
    connection: QuicConnection,
    mut evt_rx: tokio::sync::mpsc::UnboundedReceiver<WorldEvent>,
) -> anyhow::Result<()> {
    while let Some(event) = evt_rx.recv().await {
        match &event {
            WorldEvent::PlayerMoved { .. } => {
                let bytes = serialize(&event);
                connection.send_datagram(bytes.into())?;
            }
            _ => {
                let bytes = serialize(&event);
                let mut send = connection.open_uni().await?;
                send.write_all(&bytes).await?;
                send.finish()?;
            }
        }
    }
    Ok(())
}

pub fn serialize<T: Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap()
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
    Ok(postcard::from_bytes(bytes)?)
}
