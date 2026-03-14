use std::collections::HashSet;

use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

use crate::terrain::{CHUNK_RENDER_DISTANCE, chunk_in_range};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct PlayerInput {
    pub dir: Vec3,
    pub look: Vec3,
    pub sprint: bool,
}

pub struct PlayerState {
    pub pos: Vec3,
    pub look: Vec3,
    pub input: PlayerInput,
    pub chunk_anchor: Option<IVec3>,
    pub loaded_chunks: HashSet<IVec3>,
    pub name: String,
}

impl PlayerState {
    pub fn new(name: String) -> Self {
        Self {
            pos: Vec3::new(0.0, 60.0, 0.0),
            look: Vec3::default(),
            input: PlayerInput::default(),
            chunk_anchor: None,
            loaded_chunks: HashSet::new(),
            name,
        }
    }

    pub fn needs_chunk(&self, chunk_pos: IVec3) -> bool {
        match self.chunk_anchor {
            Some(anchor) => chunk_in_range(anchor, chunk_pos, CHUNK_RENDER_DISTANCE),
            None => false,
        }
    }
}
