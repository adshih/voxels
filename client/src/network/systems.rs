use bevy::prelude::*;

use server::{Message, Server};

use crate::Settings;
use crate::network::{Connection, LocalClientId, PlayerEntities, TokioRuntime, remote};
use crate::player::{LocalPlayer, RemotePlayer};

pub fn setup_connection(mut commands: Commands, settings: Res<Settings>) {
    let addr = match &settings.server_addr {
        Some(addr) => addr.parse().expect("Invalid server address"),
        None => {
            // Spawn embedded server
            let rt = tokio::runtime::Runtime::new().unwrap();
            let server = rt
                .block_on(Server::bind("127.0.0.1:0"))
                .expect("Failed to bind server");
            let addr = server.local_addr();

            std::thread::spawn(move || {
                rt.block_on(async {
                    let mut server = server;
                    server.run().await.unwrap();
                });
            });

            addr
        }
    };

    let (connection, runtime) =
        remote::connect(addr, settings.player_name.clone()).expect("Failed to connect");

    commands.insert_resource(connection);
    commands.insert_resource(TokioRuntime(runtime));
}

pub fn receive_updates(
    mut commands: Commands,
    mut connection: ResMut<Connection>,
    mut players: ResMut<PlayerEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_id: Option<Res<LocalClientId>>,
    mut local_player_transform: Single<&mut Transform, With<LocalPlayer>>,
) {
    while let Some(msg) = connection.try_recv() {
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
                pos,
                look: _,
            } => {
                if let Some(local) = &local_id
                    && local.0 == client_id
                {
                    local_player_transform.translation = pos;
                    continue;
                }

                if let Some(&entity) = players.map.get(&client_id)
                    && let Ok(mut entity_commands) = commands.get_entity(entity)
                {
                    entity_commands.insert(Transform::from_translation(pos));
                }
            }
            _ => {}
        }
    }
}

pub fn send_player_input(connection: Res<Connection>, local_player: Single<&LocalPlayer>) {
    connection.send(Message::Input {
        input: local_player.input.clone(),
    });
}
