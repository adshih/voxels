use crate::PlayerInput;

pub enum WorldCommand {
    PlayerMove { id: u32, input: PlayerInput },
}
