use bevy::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};

const CHUNK_SIZE: usize = 32;
const CHUNK_VOLUME: usize = CHUNK_SIZE.pow(3);

struct ChunkManager {
    chunks: HashMap<IVec3, Chunk>,
    pending_ops: VecDeque<ChunkOperation>,
    visible: HashSet<IVec3>,
}

#[derive(Copy, Clone, Eq, PartialEq)]
struct VoxelType(pub u8);

impl VoxelType {
    pub const AIR: VoxelType = VoxelType(0);
    pub const DIRT: VoxelType = VoxelType(1);
    pub const STONE: VoxelType = VoxelType(2);
}

struct Chunk {
    voxels: Box<[VoxelType; CHUNK_VOLUME]>,
    mesh: Option<Handle<Mesh>>,
    dirty: bool,
}

#[derive(Copy, Clone)]
enum ChunkOperation {
    Load(IVec3),
    Unload(IVec3),
}
