use bevy::prelude::*;

use net::Message;
use shared::PlayerInput;
use tokio::runtime::Runtime;

use crate::player::{LocalPlayer, RemotePlayer};

use super::client::NetworkClient;
use super::resources::*;

pub fn setup_network(mut commands: Commands) {
    let rt = Runtime::new().expect("Failed to create tokio runtime");

    let network_client = rt.block_on(async {
        NetworkClient::connect("127.0.0.1:8080", "Player1".to_string())
            .await
            .expect("Failed to connect")
    });

    commands.insert_resource(TokioRuntime(rt));
    commands.insert_resource(network_client);
}

pub fn handle_network_events(
    mut commands: Commands,
    mut network: ResMut<NetworkClient>,
    mut players: ResMut<PlayerEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_id: Option<Res<LocalClientId>>,
    mut local_player_transform: Single<&mut Transform, With<LocalPlayer>>,
) {
    while let Ok(msg) = network.event_rx.try_recv() {
        match msg {
            Message::ConnectAck { client_id } => {
                println!("Connected with id: {}", client_id);
                commands.insert_resource(LocalClientId(client_id));
            }
            Message::PlayerJoined { client_id, name } => {
                if let Some(local) = &local_id {
                    if local.0 == client_id {
                        continue;
                    }
                }

                println!("{} joined", name);
                let entity = commands
                    .spawn((
                        Name::new(format!("RemotePlayer_{}", client_id)),
                        RemotePlayer {
                            id: client_id,
                            name,
                        },
                        Transform::from_xyz(0.0, 60.0, 0.0),
                        Mesh3d(meshes.add(Capsule3d::default())),
                        MeshMaterial3d(materials.add(Color::WHITE)),
                    ))
                    .id();

                players.map.insert(client_id, entity);
            }
            Message::PlayerLeft { client_id, name: _ } => {
                println!("{} left", client_id);
                if let Some(entity) = players.map.remove(&client_id) {
                    commands.entity(entity).despawn();
                }
            }
            Message::PositionUpdate {
                client_id,
                x,
                y,
                z,
                camera_forward: _,
            } => {
                if let Some(local) = &local_id {
                    if local.0 == client_id {
                        local_player_transform.translation = Vec3::new(x, y, z);
                        continue;
                    }
                }

                if let Some(&entity) = players.map.get(&client_id) {
                    if let Ok(mut entity_commands) = commands.get_entity(entity) {
                        entity_commands.insert(Transform::from_xyz(x, y, z));
                    }
                }
            }
            _ => {}
        }
    }
}

pub fn send_input(network: Res<NetworkClient>, input: Single<&PlayerInput, With<LocalPlayer>>) {
    let msg = Message::Input { input: **input };
    let _ = network.command_tx.send(msg);
}
