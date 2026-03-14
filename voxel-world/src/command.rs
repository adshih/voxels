use serde::{Deserialize, Serialize};

use crate::PlayerInput;

#[derive(Serialize, Deserialize)]
pub struct MovePlayer {
    pub input: PlayerInput,
}

#[derive(Serialize, Deserialize)]
pub enum WorldCommand {
    MovePlayer(MovePlayer),
    Disconnect,
}

impl From<MovePlayer> for WorldCommand {
    fn from(cmd: MovePlayer) -> Self {
        Self::MovePlayer(cmd)
    }
}
