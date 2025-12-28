mod cert;
pub mod events;
mod remote;

use crate::Settings;
use crate::network::events::*;
use bevy::prelude::*;
use std::net::SocketAddr;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use voxel_net::message::{ClientMessage, ServerMessage};
use voxel_net::{Server, configure_server};

#[derive(Resource)]
pub struct TokioRuntime(#[allow(dead_code)] pub Runtime);

#[derive(Resource)]
pub struct Connection {
    outgoing: mpsc::UnboundedSender<ClientMessage>,
    incoming: mpsc::UnboundedReceiver<ServerMessage>,
}

impl Connection {
    pub fn send(&self, msg: ClientMessage) {
        let _ = self.outgoing.send(msg);
    }

    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        self.incoming.try_recv().ok()
    }
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_connection)
            .add_systems(Update, dispatch_network_messages);
    }
}

fn setup_connection(mut commands: Commands, settings: Res<Settings>) {
    let addr = match &settings.server_addr {
        Some(addr) => addr.parse().expect("Invalid server address"),
        None => spawn_embedded_server().expect("Failed to start embedded server"),
    };

    let (connection, runtime) =
        remote::connect(addr, settings.player_name.clone()).expect("Failed to connect");

    commands.insert_resource(connection);
    commands.insert_resource(TokioRuntime(runtime));
}

fn spawn_embedded_server() -> anyhow::Result<SocketAddr> {
    let rt = tokio::runtime::Runtime::new()?;
    let config = configure_server()?;
    let mut server = rt.block_on(Server::bind("127.0.0.1:0".parse().unwrap(), config))?;
    let addr = server.local_addr();

    std::thread::spawn(move || {
        rt.block_on(server.run()).unwrap();
    });

    Ok(addr)
}

fn dispatch_network_messages(mut commands: Commands, mut connection: ResMut<Connection>) {
    while let Some(msg) = connection.try_recv() {
        match msg {
            ServerMessage::ConnectAck { id, name } => commands.trigger(Connected { id, name }),
            ServerMessage::PlayerJoined { id, name } => commands.trigger(PlayerJoined { id, name }),
            ServerMessage::PlayerLeft { id, name } => commands.trigger(PlayerLeft { id, name }),
            ServerMessage::PositionUpdate { id, pos, look } => {
                commands.trigger(PositionUpdate { id, pos, look })
            }
            ServerMessage::ChunkLoaded { pos, data } => commands.trigger(ChunkLoaded { pos, data }),
            ServerMessage::ChunkUnloaded { pos } => commands.trigger(ChunkUnloaded { pos }),
        }
    }
}
