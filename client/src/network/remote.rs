use quinn::crypto::rustls::QuicClientConfig;
use quinn::{ClientConfig, Connection as QuicConnection, Endpoint, rustls};
use tokio::runtime::Runtime;

use std::net::SocketAddr;
use std::sync::Arc;

use server::Message;

use tokio::sync::mpsc;

use crate::network::cert::SkipServerVerification;

use super::Connection;

const SERVER_NAME: &str = "localhost";
const MAX_CHUNK_SIZE: usize = 70_000; // 64kb + some overhead

pub fn connect(addr: SocketAddr, player_name: String) -> anyhow::Result<(Connection, Runtime)> {
    let rt = Runtime::new()?;

    let connection = rt.block_on(async {
        let config = configure_client()?;
        let endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;

        let conn = endpoint.connect_with(config, addr, SERVER_NAME)?.await?;

        let (outgoing_tx, outgoing_rx) = mpsc::unbounded_channel();
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();

        let (mut send, mut recv) = conn.open_bi().await?;
        let connect_msg = Message::Connect { name: player_name };

        let bytes = connect_msg.serialize()?;
        send.write_all(&bytes).await?;
        send.finish()?;

        let mut buf = vec![0u8; 1024];
        if let Some(n) = recv.read(&mut buf).await? {
            if let Ok(msg) = Message::deserialize(&buf[..n]) {
                let _ = incoming_tx.send(msg);
            }
        }

        tokio::spawn(network_task(conn, outgoing_rx, incoming_tx));

        Ok::<_, anyhow::Error>(Connection {
            outgoing: outgoing_tx,
            incoming: incoming_rx,
        })
    })?;

    Ok((connection, rt))
}

fn configure_client() -> anyhow::Result<ClientConfig> {
    let crypto = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    Ok(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        crypto,
    )?)))
}

async fn network_task(
    conn: QuicConnection,
    mut outgoing_rx: mpsc::UnboundedReceiver<Message>,
    incoming_tx: mpsc::UnboundedSender<Message>,
) {
    loop {
        tokio::select! {
            result = conn.read_datagram() => {
                let Ok(data) = result else { break };
                if let Ok(msg) = Message::deserialize(&data) {
                    let _ = incoming_tx.send(msg);
                }
            }

            result = conn.accept_uni() => {
                let Ok(mut recv) = result else { break };
                let tx = incoming_tx.clone();

                tokio::spawn(async move {
                    if let Ok(data) = recv.read_to_end(MAX_CHUNK_SIZE).await {
                        if let Ok(msg) = Message::deserialize(&data) {
                            let _ = tx.send(msg);
                        }
                    }
                });
            }

            Some(msg) = outgoing_rx.recv() => {
                match &msg {
                    Message::Input { .. } => {
                        if let Ok(bytes) = msg.serialize() {
                            let _ = conn.send_datagram(bytes.into());
                        }
                    }
                    _ => {
                        if let Ok(mut send) = conn.open_uni().await {
                            if let Ok(bytes) = msg.serialize() {
                                let _ = send.write_all(&bytes).await;
                                let _ = send.finish();
                            }
                        }
                    }
                }
            }
        }
    }
}
