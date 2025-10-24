use net::Message;
use std::{io, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, signal, time::interval};

#[tokio::main]
async fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    let server_addr = "127.0.0.1:8080";

    socket.connect(server_addr).await?;
    let socket = Arc::new(socket);
    println!("Connected to server at {}", server_addr);

    let player_name = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "TestPlayer".to_string());
    let connect_msg = Message::Connect {
        name: player_name.clone(),
    };
    let bytes = connect_msg.serialize()?;

    socket.send(&bytes).await?;
    println!("Sent connect request as '{}'", player_name);

    let socket_clone = Arc::clone(&socket);
    tokio::spawn(async move {
        let mut tick = interval(Duration::from_secs(1));
        loop {
            tick.tick().await;
            let heartbeat = Message::Heartbeat;

            if let Ok(bytes) = heartbeat.serialize() {
                let _ = socket_clone.send(&bytes).await;
            }
        }
    });

    let socket_clone = Arc::clone(&socket);
    tokio::spawn(async move {
        signal::ctrl_c().await.ok();
        println!("\nDisconnecting...");

        let disconnect = Message::Disconnect;
        if let Ok(bytes) = disconnect.serialize() {
            let _ = socket_clone.send(&bytes).await;
        }

        std::process::exit(0);
    });

    let mut buf = vec![0u8; 1024];
    loop {
        let len = socket.recv(&mut buf).await?;

        match Message::deserialize(&buf[..len]) {
            Ok(msg) => match msg {
                Message::ConnectAck { client_id } => {
                    println!("Connected (id: {})", client_id);
                }
                Message::PlayerJoined { client_id: _, name } => {
                    println!("{} joined", name);
                }
                Message::PlayerLeft { client_id: _, name } => {
                    println!("{} left", name);
                }
                _ => println!("Received: {:?}", msg),
            },
            Err(e) => println!("Failed to parse message: {}", e),
        }
    }
}
