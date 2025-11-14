use shared::Message;
use tokio::runtime::Runtime;

use std::io;
use std::sync::Arc;
use std::time::Duration;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use super::Server;

struct RemoteServer {
    command_tx: mpsc::UnboundedSender<Message>,
    event_rx: mpsc::UnboundedReceiver<Message>,
}

impl RemoteServer {
    async fn connect(server_addr: &str, player_name: String) -> io::Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket.connect(server_addr).await?;
        let socket = Arc::new(socket);

        let (command_tx, command_rx) = mpsc::unbounded_channel();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let connect_msg = Message::Connect { name: player_name };
        socket.send(&connect_msg.serialize()?).await?;

        tokio::spawn(network_task(socket, command_rx, event_tx));

        Ok(RemoteServer {
            command_tx,
            event_rx,
        })
    }
}

pub fn create_remote_server(
    server_addr: &str,
    player_name: String,
) -> io::Result<(Server, Runtime)> {
    let rt = Runtime::new()?;

    let remote = rt.block_on(async { RemoteServer::connect(server_addr, player_name).await })?;

    Ok((Server::new(remote.command_tx, remote.event_rx), rt))
}

async fn network_task(
    socket: Arc<UdpSocket>,
    mut command_rx: mpsc::UnboundedReceiver<Message>,
    event_tx: mpsc::UnboundedSender<Message>,
) {
    let socket_clone = Arc::clone(&socket);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            let heartbeat = Message::Heartbeat;

            if let Ok(bytes) = heartbeat.serialize() {
                let _ = socket_clone.send(&bytes).await;
            }
        }
    });

    let mut buf = vec![0u8; 1024];

    loop {
        tokio::select! {
            result = socket.recv(&mut buf) => {
                match result {
                    Ok(len) => {
                        if let Ok(msg) = Message::deserialize(&buf[..len]) {
                            let _ = event_tx.send(msg);
                        }
                    }
                    Err(e) => {
                        eprintln!("Socket error: {}", e);
                        break;
                    }
                }
            }

            Some(msg) = command_rx.recv() => {
                let is_disconnect = matches!(msg, Message::Disconnect);

                if let Ok(bytes) = msg.serialize() {
                    let _ = socket.send(&bytes).await;
                }

                if is_disconnect {
                    break;
                }
            }
        }
    }
}
