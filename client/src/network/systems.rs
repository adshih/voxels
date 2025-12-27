use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;

use bevy::prelude::*;

use server::{Message, Server, configure_server};
use voxel_core::VoxelBuffer;

use crate::Settings;
use crate::network::{
    ChunkEntities, Connection, LocalClientId, PlayerEntities, TokioRuntime, remote,
};
use crate::player::{LocalPlayer, RemotePlayer};
use crate::world::{ChunkData, NeedsMesh};

const MAX_CHUNK_LOAD_PER_FRAME: usize = 20;

pub fn setup_connection(mut commands: Commands, settings: Res<Settings>) {
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

#[derive(Resource, Default)]
pub struct ChunkLoadQueue(pub VecDeque<(IVec3, Arc<VoxelBuffer>)>);

#[derive(Resource, Default)]
pub struct ChunkUnloadQueue(pub Vec<IVec3>);

pub fn receive_updates(
    mut commands: Commands,
    mut connection: ResMut<Connection>,
    mut players: ResMut<PlayerEntities>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    local_id: Option<Res<LocalClientId>>,
    mut local_player_transform: Single<&mut Transform, With<LocalPlayer>>,
    mut chunk_load_queue: ResMut<ChunkLoadQueue>,
    mut chunk_unload_queue: ResMut<ChunkUnloadQueue>,
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

                players.0.insert(client_id, entity);
            }
            Message::PlayerLeft { client_id, name: _ } => {
                println!("{} left", client_id);
                if let Some(entity) = players.0.remove(&client_id) {
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

                if let Some(&entity) = players.0.get(&client_id)
                    && let Ok(mut entity_commands) = commands.get_entity(entity)
                {
                    entity_commands.insert(Transform::from_translation(pos));
                }
            }
            Message::ChunkLoaded { pos, data } => {
                chunk_load_queue.0.push_back((pos, data));
            }
            Message::ChunkUnloaded { pos } => {
                chunk_load_queue.0.retain(|(p, _)| *p != pos);
                chunk_unload_queue.0.push(pos);
            }
            _ => {}
        }
    }
}

pub fn process_chunk_load_queue(
    mut commands: Commands,
    mut chunk_load_queue: ResMut<ChunkLoadQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
) {
    for _ in 0..MAX_CHUNK_LOAD_PER_FRAME {
        let Some((pos, data)) = chunk_load_queue.0.pop_front() else {
            break;
        };

        let world_pos = pos.as_vec3() * data.size.as_vec3();

        let entity = commands
            .spawn((
                Transform::from_translation(world_pos),
                ChunkData(data),
                NeedsMesh,
            ))
            .id();

        chunk_entities.0.insert(pos, entity);
    }
}

pub fn process_chunk_unload_queue(
    mut commands: Commands,
    mut chunk_unload_queue: ResMut<ChunkUnloadQueue>,
    mut chunk_entities: ResMut<ChunkEntities>,
) {
    for pos in chunk_unload_queue.0.drain(..) {
        if let Some(entity) = chunk_entities.0.remove(&pos) {
            commands.entity(entity).despawn();
        }
    }
}

pub fn send_player_input(connection: Res<Connection>, local_player: Single<&LocalPlayer>) {
    connection.send(Message::Input {
        input: local_player.input.clone(),
    });
}
