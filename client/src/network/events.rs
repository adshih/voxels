use bevy::prelude::*;
use std::sync::Arc;
use voxel_core::VoxelBuffer;

#[derive(Event)]
pub struct Connected {
    pub id: u32,
    pub name: String,
}

#[derive(Event)]
pub struct PlayerJoined {
    pub id: u32,
    pub name: String,
}

#[derive(Event)]
pub struct PlayerLeft {
    pub id: u32,
    pub name: String,
}

#[allow(dead_code)]
#[derive(Event)]
pub struct PositionUpdate {
    pub id: u32,
    pub pos: Vec3,
    pub look: Vec3,
}

#[derive(Event)]
pub struct ChunkLoaded {
    pub pos: IVec3,
    pub data: Arc<VoxelBuffer>,
}

#[derive(Event)]
pub struct ChunkUnloaded {
    pub pos: IVec3,
}
