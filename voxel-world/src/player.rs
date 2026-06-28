use std::collections::HashSet;

use glam::IVec3;
use serde::{Deserialize, Serialize};

use crate::{
    physics::BodyHandle,
    terrain::{CHUNK_RENDER_DISTANCE, chunk_in_range},
};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct PlayerInput {
    pub dir: [f32; 3],
    pub look: [f32; 3],
    pub sprint: bool,
}

#[derive(Default)]
pub struct ChunkInterest {
    pub anchor: Option<IVec3>,
    pub loaded: HashSet<IVec3>,
}

impl ChunkInterest {
    pub fn needs(&self, chunk_pos: IVec3) -> bool {
        match self.anchor {
            Some(anchor) => chunk_in_range(anchor, chunk_pos, CHUNK_RENDER_DISTANCE),
            None => false,
        }
    }
}

pub struct PlayerState {
    pub input: PlayerInput,
    pub chunks: ChunkInterest,
    pub name: String,
    pub body: BodyHandle,
}

impl PlayerState {
    pub fn new(name: String, body: BodyHandle) -> Self {
        Self {
            input: PlayerInput::default(),
            chunks: ChunkInterest::default(),
            name,
            body,
        }
    }
}
