use bevy::math::Vec3;
use shared::Message;
use shared::{PlayerInput, calculate_movement};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};
use tokio::net::UdpSocket;
use tokio::time::interval;

const TIMEOUT_DURATION: Duration = Duration::from_secs(5);

const TICK_RATE_MS: u64 = 50;
const TICK_DELTA: f32 = TICK_RATE_MS as f32 / 1000.0;

struct ClientInfo {
    addr: SocketAddr,
    id: u32,
    name: String,
    last_seen: Instant,
    position: Vec3,
    latest_input: PlayerInput,
}

struct Server {
    socket: UdpSocket,
    clients: HashMap<u32, ClientInfo>,
    addr_to_id: HashMap<SocketAddr, u32>,
    next_id: u32,
    buf: Vec<u8>,
}

impl Server {
    async fn new(addr: &str) -> std::io::Result<Self> {
        let socket = UdpSocket::bind(addr).await?;
        println!("Listening on: {}", socket.local_addr()?);

        Ok(Self {
            socket,
            clients: HashMap::new(),
            addr_to_id: HashMap::new(),
            next_id: 1,
            buf: vec![0u8; 1024],
        })
    }

    async fn run(&mut self) -> std::io::Result<()> {
        let mut timeout_check = interval(Duration::from_secs(1));
        let mut simulation_tick = interval(Duration::from_millis(TICK_RATE_MS));

        loop {
            tokio::select! {
                result = self.socket.recv_from(&mut self.buf) => {
                    let (len, addr) = result?;
                    match Message::deserialize(&self.buf[..len]) {
                        Ok(msg) => self.handle_message(msg, addr).await?,
                        Err(e) => println!("Failed to parse message from {}: {}", addr, e),
                    }
                }

                _ = timeout_check.tick() => {
                    self.check_timeouts().await?;
                }

                _ = simulation_tick.tick() => {
                    self.simulate_world(TICK_DELTA).await?;
                }
            }
        }
    }

    async fn simulate_world(&mut self, delta_time: f32) -> std::io::Result<()> {
        for client in self.clients.values_mut() {
            let forward = client.latest_input.camera_forward;
            client.position =
                calculate_movement(&client.latest_input, client.position, forward, delta_time);
        }

        for client in self.clients.values() {
            let pos_update = Message::PositionUpdate {
                client_id: client.id,
                x: client.position.x,
                y: client.position.y,
                z: client.position.z,
                camera_forward: client.latest_input.camera_forward,
            };
            self.broadcast(&pos_update).await?;
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
                if let Some(id) = self.addr_to_id.get(&addr) {
                    if let Some(client) = self.clients.get_mut(id) {
                        client.last_seen = Instant::now();
                        client.latest_input = input;
                    }
                }
            }
            _ => {
                if let Some(id) = self.addr_to_id.get(&addr) {
                    if let Some(client) = self.clients.get_mut(id) {
                        client.last_seen = Instant::now();
                    }
                }
            }
        }

        Ok(())
    }

    async fn handle_connect(&mut self, name: String, addr: SocketAddr) -> std::io::Result<()> {
        let client_id = self.next_id;
        self.next_id += 1;

        println!("{} (id: {}) connected from {}", name, client_id, addr);

        let ack = Message::ConnectAck { client_id };
        self.send_to(&ack, addr).await?;

        // Send existing players to new client.
        // TODO: send this information in a WorldState message
        for existing_client in self.clients.values() {
            let player_joined = Message::PlayerJoined {
                client_id: existing_client.id,
                name: existing_client.name.clone(),
            };
            self.send_to(&player_joined, addr).await?;
        }

        let joined_msg = Message::PlayerJoined {
            client_id,
            name: name.clone(),
        };
        self.broadcast(&joined_msg).await?;

        self.clients.insert(
            client_id,
            ClientInfo {
                addr,
                id: client_id,
                name,
                last_seen: Instant::now(),
                position: Vec3::new(0.0, 60.0, 0.0),
                latest_input: PlayerInput::default(),
            },
        );
        self.addr_to_id.insert(addr, client_id);

        Ok(())
    }

    async fn remove_client(&mut self, client_id: u32) -> std::io::Result<()> {
        if let Some(client) = self.clients.remove(&client_id) {
            self.addr_to_id.remove(&client.addr);
            println!("{} (id: {}) disconnected", client.name, client_id);

            let left_msg = Message::PlayerLeft {
                client_id,
                name: client.name,
            };
            self.broadcast(&left_msg).await?;
        }

        Ok(())
    }

    async fn check_timeouts(&mut self) -> std::io::Result<()> {
        let now = Instant::now();
        let mut disconnected = Vec::new();

        for (id, client) in &self.clients {
            if now.duration_since(client.last_seen) > TIMEOUT_DURATION {
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut server = Server::new("127.0.0.1:8080").await?;
    server.run().await
}
