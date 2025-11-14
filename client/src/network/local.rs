use bevy::prelude::*;
use shared::{Message, PlayerInput, calculate_movement};
use tokio::sync::mpsc;

use super::Server;

#[derive(Resource)]
pub struct LocalServer {
    command_rx: mpsc::UnboundedReceiver<Message>,
    event_tx: mpsc::UnboundedSender<Message>,
    position: Vec3,
    latest_input: PlayerInput,
    client_id: u32,
}

pub fn create_local_server() -> (Server, LocalServer) {
    let (command_tx, command_rx) = mpsc::unbounded_channel();
    let (event_tx, event_rx) = mpsc::unbounded_channel();

    let server = Server::new(command_tx, event_rx);

    let local_server = LocalServer {
        command_rx,
        event_tx,
        position: Vec3::new(0.0, 60.0, 0.0),
        latest_input: PlayerInput::default(),
        client_id: 1,
    };

    let _ = local_server
        .event_tx
        .send(Message::ConnectAck { client_id: 1 });

    (server, local_server)
}

pub fn run_local_simulation(mut local_server: ResMut<LocalServer>, time: Res<Time>) {
    while let Ok(msg) = local_server.command_rx.try_recv() {
        match msg {
            Message::Input { input } => {
                local_server.latest_input = input;
            }
            Message::Disconnect => {}
            _ => {}
        }
    }

    let camera_forward = local_server.latest_input.camera_forward;
    local_server.position = calculate_movement(
        &local_server.latest_input,
        local_server.position,
        camera_forward,
        time.delta_secs(),
    );

    let pos_update = Message::PositionUpdate {
        client_id: local_server.client_id,
        x: local_server.position.x,
        y: local_server.position.y,
        z: local_server.position.z,
        camera_forward: local_server.latest_input.camera_forward,
    };

    let _ = local_server.event_tx.send(pos_update);
}
