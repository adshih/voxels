use glam::{IVec3, Vec3};
use serde::{Deserialize, Serialize};

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
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 60.0, 0.0),
            look: Vec3::default(),
            input: PlayerInput::default(),
            chunk_anchor: None,
        }
    }
}
