use std::sync::Arc;

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};
use voxel_core::VoxelBuffer;
use voxel_world::player::PlayerInput;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    Connect {
        name: String,
    },
    ConnectAck {
        client_id: u32,
    },
    PlayerJoined {
        client_id: u32,
        name: String,
    },
    PlayerLeft {
        client_id: u32,
        name: String,
    },
    Heartbeat,
    Disconnect,
    Input {
        input: PlayerInput,
    },
    PositionUpdate {
        client_id: u32,
        pos: Vec3,
        look: Vec3,
    },
    ChunkLoaded {
        pos: IVec3,
        data: Arc<VoxelBuffer>,
    },
    ChunkUnloaded {
        pos: IVec3,
    },
}

impl Message {
    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        let bytes = postcard::to_allocvec(self)?;
        Ok(bytes)
    }

    pub fn deserialize(bytes: &[u8]) -> anyhow::Result<Self> {
        let msg = postcard::from_bytes(bytes)?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_roundtrip() {
        let msg = Message::Connect {
            name: "Adam".to_string(),
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded {
            Message::Connect { name } => assert_eq!(name, "Adam"),
            _ => panic!("Wrong message type"),
        }
    }
}
