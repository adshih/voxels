use crate::{
    network::{ChunkLoadQueue, ChunkUnloadQueue},
    world::{MAX_CHUNK_LOAD_PER_FRAME, NeedsMesh},
};
use bevy::prelude::*;
use std::{collections::HashMap, sync::Arc};
use voxel_core::VoxelBuffer;

#[derive(Component)]
pub struct ChunkData(pub Arc<VoxelBuffer>);

#[derive(Default, Resource)]
pub struct ChunkEntities(pub HashMap<IVec3, Entity>);

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
