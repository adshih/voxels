use serde::{Deserialize, Serialize, de::DeserializeOwned};
use voxel_world::{
    command::{MovePlayer, WorldCommand},
    request::Connect,
};

#[derive(Serialize, Deserialize)]
pub enum ClientRequest {
    Connect(Connect),
    Ping,
}

#[derive(Serialize, Deserialize)]
pub enum ClientCommand {
    MovePlayer(MovePlayer),
}

impl From<ClientCommand> for WorldCommand {
    fn from(cmd: ClientCommand) -> Self {
        match cmd {
            ClientCommand::MovePlayer(input) => WorldCommand::MovePlayer(input),
        }
    }
}

pub fn serialize<T: Serialize>(value: &T) -> Vec<u8> {
    postcard::to_allocvec(value).unwrap()
}

pub fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
    Ok(postcard::from_bytes(bytes)?)
}
