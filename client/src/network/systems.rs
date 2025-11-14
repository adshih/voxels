use bevy::prelude::*;
use shared::{Message, PlayerInput};

use crate::network::{LocalClientId, PlayerEntities, TokioRuntime};
use crate::player::{LocalPlayer, RemotePlayer};

use super::Server;
use super::local::create_local_server;
use super::remote::create_remote_server;

pub fn setup_server(mut commands: Commands, settings: Res<crate::Settings>) {
    if settings.multiplayer {
        let (server, runtime) = create_remote_server("127.0.0.1:8080", "Player1".to_string())
            .expect("Failed to connect to server");

        commands.insert_resource(server);
        commands.insert_resource(TokioRuntime(runtime));
    } else {
        let (server, local_server) = create_local_server();

        commands.insert_resource(server);
        commands.insert_resource(local_server);
    }
}
pub fn receive_updates(
    mut commands: Commands,
    mut server: ResMut<Server>,
    mut players: ResMut<PlayerEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_id: Option<Res<LocalClientId>>,
    mut local_player_transform: Single<&mut Transform, With<LocalPlayer>>,
) {
    while let Some(msg) = server.try_recv() {
        match msg {
            Message::ConnectAck { client_id } => {
                println!("Connected with id: {}", client_id);
                commands.insert_resource(LocalClientId(client_id));
            }
            Message::PlayerJoined { client_id, name } => {
                if let Some(local) = &local_id
                    && local.0 == client_id
                {
                    continue;
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
                if let Some(local) = &local_id
                    && local.0 == client_id
                {
                    local_player_transform.translation = Vec3::new(x, y, z);
                    continue;
                }

                if let Some(&entity) = players.map.get(&client_id)
                    && let Ok(mut entity_commands) = commands.get_entity(entity)
                {
                    entity_commands.insert(Transform::from_xyz(x, y, z));
                }
            }
            _ => {}
        }
    }
}

pub fn send_player_input(server: Res<Server>, input: Single<&PlayerInput, With<LocalPlayer>>) {
    let msg = Message::Input { input: **input };
    server.send(msg);
}
