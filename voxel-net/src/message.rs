use std::sync::Arc;

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use voxel_core::VoxelBuffer;
use voxel_world::player::PlayerInput;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    Connect { name: String },
    Input { input: PlayerInput },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    ConnectAck { id: u32, name: String },
    PlayerJoined { id: u32, name: String },
    PlayerLeft { id: u32, name: String },
    PositionUpdate { id: u32, pos: Vec3, look: Vec3 },
    ChunkLoaded { pos: IVec3, data: Arc<VoxelBuffer> },
    ChunkUnloaded { pos: IVec3 },
}

pub trait WireMessage: Sized {
    fn serialize(&self) -> anyhow::Result<Vec<u8>>;
    fn deserialize(bytes: &[u8]) -> anyhow::Result<Self>;
}

impl<T> WireMessage for T
where
    T: Serialize + for<'de> Deserialize<'de>,
{
    fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = postcard::to_allocvec(self)?;
        Ok(bytes)
    }

    fn deserialize(bytes: &[u8]) -> anyhow::Result<Self> {
        let msg = postcard::from_bytes(bytes)?;
        Ok(msg)
    }
}
