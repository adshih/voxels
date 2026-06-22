use std::collections::HashSet;

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

use crate::terrain::{CHUNK_RENDER_DISTANCE, chunk_in_range};

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
    pub pos: Vec3,
    pub look: Vec3,
    pub input: PlayerInput,
    pub chunks: ChunkInterest,
    pub name: String,
}

impl PlayerState {
    pub fn new(name: String) -> Self {
        Self {
            pos: Vec3::new(0.0, 60.0, 0.0),
            look: Vec3::default(),
            input: PlayerInput::default(),
            chunks: ChunkInterest::default(),
            name,
        }
    }
}
