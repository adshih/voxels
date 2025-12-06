use tokio::runtime::Runtime;

use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use server::Message;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use super::Connection;

pub fn connect(addr: SocketAddr, player_name: String) -> io::Result<(Connection, Runtime)> {
    let rt = Runtime::new()?;

    let connection = rt.block_on(async {
        let socket = Arc::new(UdpSocket::bind("0.0.0.0:0").await?);
        socket.connect(addr).await?;

        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();

        let connect_msg = Message::Connect { name: player_name };
        socket.send(&connect_msg.serialize()?).await?;

        tokio::spawn(network_task(socket, outgoing_rx, incoming_tx));

        Ok::<_, io::Error>(Connection {
            outgoing: outgoing_tx,
            incoming: incoming_rx,
        })
    })?;

    Ok((connection, rt))
}

async fn network_task(
    socket: Arc<UdpSocket>,
    mut outgoing_rx: mpsc::UnboundedReceiver<Message>,
    incoming_tx: mpsc::UnboundedSender<Message>,
) {
    // Heartbeat task
    let socket_clone = Arc::clone(&socket);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await;
            if let Ok(bytes) = Message::Heartbeat.serialize() {
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
                            let _ = incoming_tx.send(msg);
                        }
                    }
                    Err(e) => {
                        eprintln!("Socket error: {}", e);
                        break;
                    }
                }
            }

            Some(msg) = outgoing_rx.recv() => {
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
