use std::sync::Arc;

use glam::{IVec3, Vec3};
use voxel_core::VoxelBuffer;

pub enum WorldEvent {
    PlayerMoved {
        id: u32,
        pos: Vec3,
        look: Vec3,
    },
    ChunkLoaded {
        for_player: u32,
        pos: IVec3,
        data: Arc<VoxelBuffer>,
    },
    ChunkUnloaded {
        for_player: u32,
        pos: IVec3,
    },
}
