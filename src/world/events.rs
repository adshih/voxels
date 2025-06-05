use super::ChunkCoord;
use bevy::prelude::*;

#[derive(Event)]
pub struct ChunkNeedsGeneration {
    pub entity: Entity,
    pub coord: ChunkCoord,
}

#[derive(Event)]
pub struct ChunkVoxelsReady {
    pub entity: Entity,
    pub coord: ChunkCoord,
}

#[derive(Event)]
pub struct ChunkNeedsMesh {
    pub entity: Entity,
    pub coord: ChunkCoord,
    pub priority: MeshPriority,
}

#[derive(Event)]
pub struct ChunkMeshReady {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
    pub voxel_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeshPriority {
    Normal,
}

impl Default for MeshPriority {
    fn default() -> Self {
        Self::Normal
    }
}
