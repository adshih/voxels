use std::sync::Arc;

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use voxel_core::VoxelBuffer;

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerMoved {
    pub id: u32,
    pub pos: Vec3,
    pub look: Vec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerJoined {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PlayerLeft {
    pub id: u32,
    pub name: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkLoaded {
    pub pos: IVec3,
    pub data: Arc<VoxelBuffer>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct ChunkUnloaded {
    pub pos: IVec3,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum WorldEvent {
    PlayerMoved(PlayerMoved),
    PlayerJoined(PlayerJoined),
    PlayerLeft(PlayerLeft),
    ChunkLoaded(ChunkLoaded),
    ChunkUnloaded(ChunkUnloaded),
}

impl From<PlayerMoved> for WorldEvent {
    fn from(e: PlayerMoved) -> Self {
        Self::PlayerMoved(e)
    }
}

impl From<PlayerJoined> for WorldEvent {
    fn from(e: PlayerJoined) -> Self {
        Self::PlayerJoined(e)
    }
}

impl From<PlayerLeft> for WorldEvent {
    fn from(e: PlayerLeft) -> Self {
        Self::PlayerLeft(e)
    }
}

impl From<ChunkLoaded> for WorldEvent {
    fn from(e: ChunkLoaded) -> Self {
        Self::ChunkLoaded(e)
    }
}

impl From<ChunkUnloaded> for WorldEvent {
    fn from(e: ChunkUnloaded) -> Self {
        Self::ChunkUnloaded(e)
    }
}
