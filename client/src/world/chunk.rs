use bevy::camera::primitives::Aabb;
use bevy::prelude::*;
use std::collections::hash_map::Entry;
use std::collections::{HashMap, VecDeque};
use voxel_core::VoxelBuffer;

use crate::player::LocalPlayer;

use super::WorldSettings;
use super::events::*;

pub const CHUNK_SIZE: usize = 32;

const MAX_OPERATIONS_PER_FRAME: usize = 2;

#[derive(Component)]
pub struct Chunk;

#[derive(Component, Clone)]
pub struct ChunkVoxels {
    pub data: VoxelBuffer,
    pub version: u32,
}

impl ChunkVoxels {
    pub fn new() -> Self {
        Self {
            data: VoxelBuffer::new(UVec3::splat(CHUNK_SIZE as u32)),
            version: 1,
        }
    }
}

#[derive(Component)]
pub struct ChunkMesh {
    pub handle: Handle<Mesh>,
}

#[derive(Copy, Clone, Debug)]
pub enum ChunkOperation {
    Load(IVec3),
    Unload(IVec3),
}

#[derive(Resource, Default)]
pub struct ChunkManager {
    pub pending_ops: VecDeque<ChunkOperation>,
    pub loaded_chunks: HashMap<IVec3, Entity>,
}

impl ChunkManager {
    fn queue_load(&mut self, coord: IVec3) {
        if !self.loaded_chunks.contains_key(&coord) {
            self.pending_ops.push_back(ChunkOperation::Load(coord));
        }
    }

    fn queue_unload(&mut self, coord: IVec3) {
        if self.loaded_chunks.contains_key(&coord) {
            self.pending_ops.push_back(ChunkOperation::Unload(coord));
        }
    }
}

fn from_world_pos(pos: Vec3) -> IVec3 {
    IVec3::new(
        (pos.x / CHUNK_SIZE as f32).floor() as i32,
        (pos.y / CHUNK_SIZE as f32).floor() as i32,
        (pos.z / CHUNK_SIZE as f32).floor() as i32,
    )
}

pub fn queue_chunk_operations(
    mut chunk_manager: ResMut<ChunkManager>,
    world_settings: Res<WorldSettings>,
    local_player_transform: Single<&Transform, With<LocalPlayer>>,
) {
    let player_chunk = from_world_pos(local_player_transform.translation);
    let player_pos = local_player_transform.translation;
    let render_distance = world_settings.render_distance as i32;

    let mut chunks_to_load = Vec::new();

    for x in -render_distance..=render_distance {
        for z in -render_distance..=render_distance {
            for y in 0..=15 {
                let distance_sq = x * x + z * z;
                let max_distance_sq = render_distance * render_distance;

                if distance_sq <= max_distance_sq {
                    let chunk_coord = IVec3::new(player_chunk.x + x, y, player_chunk.z + z);

                    let chunk_world_pos = Vec3::new(
                        chunk_coord.x as f32 * CHUNK_SIZE as f32,
                        chunk_coord.y as f32 * CHUNK_SIZE as f32,
                        chunk_coord.z as f32 * CHUNK_SIZE as f32,
                    );
                    let distance = player_pos.distance(chunk_world_pos);

                    chunks_to_load.push((chunk_coord, distance));
                }
            }
        }
    }

    chunks_to_load.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let unload_distance = render_distance + 2;
    let chunks_to_unload: Vec<IVec3> = chunk_manager
        .loaded_chunks
        .keys()
        .filter(|&&coord| {
            let dx = coord.x - player_chunk.x;
            let dz = coord.z - player_chunk.z;

            // let distance_sq = (coord.0 - player_chunk.0).length_squared();

            let distance_sq = dx * dx + dz * dz;

            distance_sq > (unload_distance * unload_distance)
        })
        .copied()
        .collect();

    for coord in chunks_to_unload {
        chunk_manager.queue_unload(coord);
    }

    for (coord, _distance) in chunks_to_load {
        chunk_manager.queue_load(coord);
    }
}

pub fn process_chunk_operations(
    mut commands: Commands,
    mut chunk_manager: ResMut<ChunkManager>,
    mut generation_events: MessageWriter<ChunkNeedsGeneration>,
) {
    let batch: Vec<_> = chunk_manager
        .pending_ops
        .drain(..)
        .take(MAX_OPERATIONS_PER_FRAME)
        .collect();

    for operation in batch {
        match operation {
            ChunkOperation::Load(coord) => {
                if let Entry::Vacant(e) = chunk_manager.loaded_chunks.entry(coord) {
                    let entity = commands
                        .spawn((
                            Chunk,
                            Aabb::from_min_max(Vec3::ZERO, Vec3::splat(CHUNK_SIZE as f32)),
                            Transform::from_translation(Vec3::new(
                                coord.x as f32 * CHUNK_SIZE as f32,
                                coord.y as f32 * CHUNK_SIZE as f32,
                                coord.z as f32 * CHUNK_SIZE as f32,
                            )),
                        ))
                        .id();
                    e.insert(entity);
                    generation_events.write(ChunkNeedsGeneration { entity, coord });
                }
            }
            ChunkOperation::Unload(coord) => {
                if let Some(entity) = chunk_manager.loaded_chunks.remove(&coord) {
                    commands.entity(entity).despawn();
                }
            }
        }
    }
}
