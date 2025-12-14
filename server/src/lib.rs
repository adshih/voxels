mod message;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::interval;
use voxel_world::VoxelWorld;

pub use message::Message;
use voxel_world::commands::WorldCommand;
use voxel_world::events::WorldEvent;

const TICK_RATE: f32 = 60.0;
const DT: f32 = 1.0 / TICK_RATE;
const TIMEOUT: Duration = Duration::from_secs(5);

struct ClientInfo {
    addr: SocketAddr,
    name: String,
    last_seen: Instant,
}

pub struct Server {
    socket: UdpSocket,
    buf: Vec<u8>,
    world: VoxelWorld,
    clients: HashMap<u32, ClientInfo>,
    addr_to_id: HashMap<SocketAddr, u32>,
}

impl Server {
    pub async fn bind(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!("Listening on {}", socket.local_addr()?);

        Ok(Self {
            socket,
            buf: vec![0u8; 1024],
            world: VoxelWorld::new(123),
            clients: HashMap::new(),
            addr_to_id: HashMap::new(),
        })
    }

    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap()
    }

    pub async fn run(&mut self) -> std::io::Result<()> {
        let mut tick = interval(Duration::from_secs_f32(DT));
        let mut timeout_check = interval(Duration::from_secs(1));

        loop {
            tokio::select! {
                result = self.socket.recv_from(&mut self.buf) => {
                    let (len, addr) = result?;
                    if let Ok(msg) = Message::deserialize(&self.buf[..len]) {
                        self.handle_message(msg, addr).await?;
                    }
                }

                _ = tick.tick() => {
                    self.world.tick(DT);

                    for event in self.world.drain_events() {
                        self.handle_event(event).await?;
                    }
                }

                _ = timeout_check.tick() => {
                    self.check_timeouts().await?;
                }
            }
        }
    }

    async fn handle_event(&self, event: WorldEvent) -> std::io::Result<()> {
        match event {
            WorldEvent::PlayerMoved { id, pos, look } => {
                self.broadcast(&Message::PositionUpdate {
                    client_id: id,
                    pos,
                    look,
                })
                .await?;
            }
        }

        Ok(())
    }

    async fn handle_message(&mut self, msg: Message, addr: SocketAddr) -> std::io::Result<()> {
        match msg {
            Message::Connect { name } => {
                self.handle_connect(name, addr).await?;
            }
            Message::Disconnect => {
                if let Some(id) = self.addr_to_id.get(&addr) {
                    self.remove_client(*id).await?;
                }
            }
            Message::Input { input } => {
                if let Some(&id) = self.addr_to_id.get(&addr) {
                    if let Some(client) = self.clients.get_mut(&id) {
                        client.last_seen = Instant::now();
                    }
                    self.world.execute(WorldCommand::PlayerMove { id, input });
                }
            }
            _ => {
                if let Some(id) = self.addr_to_id.get(&addr)
                    && let Some(client) = self.clients.get_mut(id)
                {
                    client.last_seen = Instant::now();
                }
            }
        }

        Ok(())
    }

    async fn handle_connect(&mut self, name: String, addr: SocketAddr) -> std::io::Result<()> {
        let id = self.world.add_player();
        println!("{} ({}) connected", name, id);

        self.send_to(&Message::ConnectAck { client_id: id }, addr)
            .await?;

        for (&id, client_info) in &self.clients {
            self.send_to(
                &Message::PlayerJoined {
                    client_id: id,
                    name: client_info.name.clone(),
                },
                addr,
            )
            .await?;
        }

        self.broadcast(&Message::PlayerJoined {
            client_id: id,
            name: name.clone(),
        })
        .await?;

        self.clients.insert(
            id,
            ClientInfo {
                addr,
                name,
                last_seen: Instant::now(),
            },
        );
        self.addr_to_id.insert(addr, id);

        Ok(())
    }

    async fn remove_client(&mut self, id: u32) -> std::io::Result<()> {
        let Some(client) = self.clients.remove(&id) else {
            return Ok(());
        };

        self.addr_to_id.remove(&client.addr);
        println!("{} ({}) disconnected", client.name, id);

        let left_msg = Message::PlayerLeft {
            client_id: id,
            name: client.name,
        };
        self.broadcast(&left_msg).await
    }

    async fn check_timeouts(&mut self) -> std::io::Result<()> {
        let now = Instant::now();
        let mut disconnected = Vec::new();

        for (id, client) in &self.clients {
            if now.duration_since(client.last_seen) > TIMEOUT {
                disconnected.push(*id);
            }
        }

        for id in disconnected {
            self.remove_client(id).await?;
        }

        Ok(())
    }

    async fn send_to(&self, msg: &Message, addr: SocketAddr) -> std::io::Result<()> {
        let bytes = msg.serialize()?;
        self.socket.send_to(&bytes, addr).await?;
        Ok(())
    }

    async fn broadcast(&self, msg: &Message) -> std::io::Result<()> {
        let bytes = msg.serialize()?;

        for client in self.clients.values() {
            self.socket.send_to(&bytes, client.addr).await?;
        }

        Ok(())
    }
}
