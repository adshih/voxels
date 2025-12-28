use crate::{
    network::events::{ChunkLoaded, ChunkUnloaded},
    world::{MAX_CHUNK_LOAD_PER_FRAME, NeedsMesh},
};
use bevy::prelude::*;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use voxel_core::VoxelBuffer;

#[derive(Component)]
pub struct ChunkData(pub Arc<VoxelBuffer>);

#[derive(Default, Resource)]
pub struct ChunkEntities(pub HashMap<IVec3, Entity>);

#[derive(Resource, Default)]
pub struct ChunkLoadQueue(pub VecDeque<(IVec3, Arc<VoxelBuffer>)>);

#[derive(Resource, Default)]
pub struct ChunkUnloadQueue(pub Vec<IVec3>);

pub fn on_chunk_loaded(on: On<ChunkLoaded>, mut queue: ResMut<ChunkLoadQueue>) {
    let event = on.event();
    queue.0.push_back((event.pos, event.data.clone()));
}

pub fn on_chunk_unloaded(
    on: On<ChunkUnloaded>,
    mut load_queue: ResMut<ChunkLoadQueue>,
    mut unload_queue: ResMut<ChunkUnloadQueue>,
) {
    let event = on.event();
    load_queue.0.retain(|(p, _)| *p != event.pos);
    unload_queue.0.push(event.pos);
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
