use std::{
    net::SocketAddr,
    sync::{Arc, OnceLock},
};

use quinn::{ClientConfig, Connection as QuicConnection, Endpoint, crypto::rustls::QuicClientConfig};
use tokio::{runtime::Runtime, sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel}};
use voxel_net::message::{ClientCommand, ClientRequest, deserialize, serialize};
use voxel_world::{Bridge, command::WorldCommand, event::WorldEvent, request::{Connect, WorldRequest}};

use crate::connection::cert::SkipServerVerification;

const SERVER_NAME: &str = "localhost";
const MAX_MSG_SIZE: usize = 1024 * 1024;

static RT: OnceLock<Runtime> = OnceLock::new();

pub fn connect(addr: String, name: String) -> anyhow::Result<(u32, Bridge)> {
    let addr: SocketAddr = addr.parse()?;
    let rt = RT.get_or_init(|| Runtime::new().unwrap());

    let (id, bridge) = rt.block_on(async {
        let config = configure_client()?;
        let mut endpoint = Endpoint::client("0.0.0.0:0".parse().unwrap())?;
        endpoint.set_default_client_config(config);

        let conn = endpoint.connect(addr, SERVER_NAME)?.await?;

        let id = handshake(&conn, &name).await?;

        let (cmd_tx, cmd_rx) = unbounded_channel();
        let (req_tx, req_rx) = unbounded_channel();
        let (event_tx, event_rx) = unbounded_channel();

        tokio::spawn(network_task(conn, cmd_rx, req_rx, event_tx));

        let bridge = Bridge::new(cmd_tx, req_tx, event_rx);
        Ok::<_, anyhow::Error>((id, bridge))
    })?;

    Ok((id, bridge))
}

fn configure_client() -> anyhow::Result<ClientConfig> {
    let crypto = quinn::rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(SkipServerVerification::new())
        .with_no_client_auth();

    Ok(ClientConfig::new(Arc::new(QuicClientConfig::try_from(
        crypto,
    )?)))
}

async fn handshake(conn: &QuicConnection, name: &str) -> anyhow::Result<u32> {
    let (mut send, mut recv) = conn.open_bi().await?;
    let connect = Connect {
        name: name.to_string(),
    };
    send.write_all(&serialize(&connect)).await?;
    send.finish()?;

    let bytes = recv.read_to_end(MAX_MSG_SIZE).await?;
    let id: u32 = deserialize(&bytes)?;
    Ok(id)
}

async fn network_task(
    conn: QuicConnection,
    mut cmd_rx: UnboundedReceiver<WorldCommand>,
    mut req_rx: UnboundedReceiver<WorldRequest>,
    event_tx: UnboundedSender<WorldEvent>,
) {
    tokio::select! {
        _ = receive_events(conn.clone(), event_tx) => (),
        _ = send_commands(conn.clone(), &mut cmd_rx) => (),
        _ = handle_requests(conn.clone(), &mut req_rx) => (),
    }
}

async fn send_commands(
    conn: QuicConnection,
    cmd_rx: &mut UnboundedReceiver<WorldCommand>,
) -> anyhow::Result<()> {
    while let Some(cmd) = cmd_rx.recv().await {
        let client_cmd = match cmd {
            WorldCommand::MovePlayer(m) => ClientCommand::MovePlayer(m),
            WorldCommand::Disconnect => break,
        };
        let bytes = serialize(&client_cmd);
        conn.send_datagram(bytes.into())?;
    }
    Ok(())
}

async fn receive_events(
    conn: QuicConnection,
    event_tx: UnboundedSender<WorldEvent>,
) -> anyhow::Result<()> {
    loop {
        tokio::select! {
            result = conn.read_datagram() => {
                let data = result?;
                let event: WorldEvent = deserialize(&data)?;
                let _ = event_tx.send(event);
            }
            result = conn.accept_uni() => {
                let mut recv = result?;
                let tx = event_tx.clone();
                tokio::spawn(async move {
                    if let Ok(data) = recv.read_to_end(MAX_MSG_SIZE).await {
                        if let Ok(event) = deserialize::<WorldEvent>(&data) {
                            let _ = tx.send(event);
                        }
                    }
                });
            }
        }
    }
}

async fn handle_requests(
    conn: QuicConnection,
    req_rx: &mut UnboundedReceiver<WorldRequest>,
) -> anyhow::Result<()> {
    while let Some(req) = req_rx.recv().await {
        match req {
            WorldRequest::Ping(call) => {
                let (mut send, mut recv) = conn.open_bi().await?;
                send.write_all(&serialize(&ClientRequest::Ping)).await?;
                send.finish()?;

                let bytes = recv.read_to_end(MAX_MSG_SIZE).await?;
                let pong = deserialize(&bytes)?;
                call.reply(pong);
            }
            _ => {}
        }
    }
    Ok(())
}
