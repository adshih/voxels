use bevy::math::Affine3A;
use bevy::prelude::*;
use bevy::render::primitives::{Aabb, Frustum};
use std::collections::hash_map::Entry;
use std::collections::{HashMap, HashSet, VecDeque};

use super::WorldSettings;
use super::events::*;
use crate::player::Player;

const VOXEL_SIZE: f32 = 1.0;

pub const CHUNK_SIZE: usize = 16;
pub const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);
pub const CHUNK_WORLD_SIZE: f32 = CHUNK_SIZE as f32 * VOXEL_SIZE;

const MAX_OPERATIONS_PER_FRAME: usize = 2;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct VoxelType(pub u8);

impl VoxelType {
    pub const AIR: VoxelType = VoxelType(0);
    pub const DIRT: VoxelType = VoxelType(1);
    pub const STONE: VoxelType = VoxelType(2);

    pub fn is_solid(self) -> bool {
        self != VoxelType::AIR
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ChunkCoord(pub IVec3);

impl ChunkCoord {
    fn from_world_pos(pos: Vec3) -> Self {
        Self(IVec3::new(
            (pos.x / CHUNK_WORLD_SIZE).floor() as i32,
            (pos.y / CHUNK_WORLD_SIZE).floor() as i32,
            (pos.z / CHUNK_WORLD_SIZE).floor() as i32,
        ))
    }

    fn _to_world_pos(self) -> Vec3 {
        Vec3::new(
            self.0.x as f32 * CHUNK_WORLD_SIZE,
            self.0.y as f32 * CHUNK_WORLD_SIZE,
            self.0.z as f32 * CHUNK_WORLD_SIZE,
        )
    }
}

#[derive(Component)]
pub struct Chunk {
    pub coord: ChunkCoord,
}

#[derive(Component, Clone)]
pub struct ChunkVoxels {
    pub data: Box<[VoxelType; CHUNK_VOLUME]>,
    pub version: u32,
}

impl ChunkVoxels {
    pub fn new() -> Self {
        Self {
            data: Box::new([VoxelType::AIR; CHUNK_VOLUME]),
            version: 1,
        }
    }

    pub fn set(&mut self, x: usize, y: usize, z: usize, voxel_type: VoxelType) {
        if let Some(i) = self.index(x, y, z) {
            if self.data[i] != voxel_type {
                self.data[i] = voxel_type;
                self.version += 1;
            }
        }
    }

    pub fn get(&self, x: usize, y: usize, z: usize) -> VoxelType {
        if let Some(i) = self.index(x, y, z) {
            self.data[i]
        } else {
            VoxelType::AIR
        }
    }

    fn index(&self, x: usize, y: usize, z: usize) -> Option<usize> {
        if x < CHUNK_SIZE && y < CHUNK_SIZE && z < CHUNK_SIZE {
            Some(x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE)
        } else {
            None
        }
    }
}

#[derive(Component)]
pub struct ChunkMesh {
    pub handle: Handle<Mesh>,
    pub voxel_version: u32,
}

#[derive(Copy, Clone)]
enum ChunkOperation {
    Load(ChunkCoord),
    Unload(ChunkCoord),
}

#[derive(Resource)]
pub struct ChunkManager {
    pending_ops: VecDeque<ChunkOperation>,
    loaded_chunks: HashMap<ChunkCoord, Entity>,
    visible_chunks: HashSet<ChunkCoord>,
}

impl Default for ChunkManager {
    fn default() -> Self {
        Self {
            pending_ops: VecDeque::new(),
            loaded_chunks: HashMap::new(),
            visible_chunks: HashSet::new(),
        }
    }
}

impl ChunkManager {
    fn queue_load(&mut self, coord: ChunkCoord) {
        if !self.loaded_chunks.contains_key(&coord) {
            self.pending_ops.push_back(ChunkOperation::Load(coord));
        }
    }

    fn queue_unload(&mut self, coord: ChunkCoord) {
        if self.loaded_chunks.contains_key(&coord) {
            self.pending_ops.push_back(ChunkOperation::Unload(coord));
        }
    }

    pub fn cull_chunks(&mut self, frustum: &Frustum, margin: f32) {
        self.visible_chunks.clear();

        for &coord in self.loaded_chunks.keys() {
            let chunk_aabb = get_chunk_aabb(coord, margin);

            if frustum.intersects_obb(&chunk_aabb, &Affine3A::IDENTITY, true, true) {
                self.visible_chunks.insert(coord);
            }
        }
    }
}

fn get_chunk_aabb(coord: ChunkCoord, margin: f32) -> Aabb {
    let min = Vec3::new(
        coord.0.x as f32 * CHUNK_WORLD_SIZE,
        coord.0.y as f32 * CHUNK_WORLD_SIZE,
        coord.0.z as f32 * CHUNK_WORLD_SIZE,
    );
    let max = min + Vec3::splat(CHUNK_WORLD_SIZE);

    let expansion = Vec3::splat(margin);
    Aabb::from_min_max(min - expansion, max + expansion)
}

pub fn update_chunk_visibility(
    mut chunk_manager: ResMut<ChunkManager>,
    mut chunks: Query<(&Chunk, &mut Visibility), With<ChunkMesh>>,
    camera_query: Query<&Frustum, With<Camera>>,
) {
    if let Ok(frustum) = camera_query.single() {
        let margin = CHUNK_WORLD_SIZE * 1.5;
        chunk_manager.cull_chunks(frustum, margin);

        for (chunk, mut visibility) in chunks.iter_mut() {
            *visibility = if chunk_manager.visible_chunks.contains(&chunk.coord) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        }
    }
}

pub fn queue_chunk_operations(
    mut chunk_manager: ResMut<ChunkManager>,
    world_settings: Res<WorldSettings>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.single().expect("Could not find player");
    let player_chunk = ChunkCoord::from_world_pos(player_transform.translation);
    let player_pos = player_transform.translation;
    let render_distance = world_settings.render_distance as i32;

    let mut chunks_to_load = Vec::new();

    for x in -render_distance..=render_distance {
        for z in -render_distance..=render_distance {
            for y in -2..=2 {
                let distance_sq = x * x + z * z + y * y;
                let max_distance_sq = render_distance * render_distance;

                if distance_sq <= max_distance_sq {
                    let chunk_coord = ChunkCoord(player_chunk.0 + IVec3::new(x, y, z));

                    let chunk_world_pos = Vec3::new(
                        chunk_coord.0.x as f32 * CHUNK_WORLD_SIZE,
                        chunk_coord.0.y as f32 * CHUNK_WORLD_SIZE,
                        chunk_coord.0.z as f32 * CHUNK_WORLD_SIZE,
                    );
                    let distance = player_pos.distance(chunk_world_pos);

                    chunks_to_load.push((chunk_coord, distance));
                }
            }
        }
    }

    chunks_to_load.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    let unload_distance = render_distance + 2;
    let chunks_to_unload: Vec<ChunkCoord> = chunk_manager
        .loaded_chunks
        .keys()
        .filter(|&&coord| {
            let distance_sq = (coord.0 - player_chunk.0).length_squared();
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
    mut generation_events: EventWriter<ChunkNeedsGeneration>,
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
                    let entity = commands.spawn(Chunk { coord }).id();
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
