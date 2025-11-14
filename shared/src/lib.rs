use bevy::prelude::*;
use std::io::{self, Cursor, Read, Write};

pub const MOVEMENT_SPEED: f32 = 10.0;
pub const SPRINT_MULTIPLIER: f32 = 2.0;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct PlayerInput {
    pub forward: f32,
    pub right: f32,
    pub up: f32,
    pub sprint: bool,
    pub camera_forward: Vec3,
}

pub fn calculate_movement(
    input: &PlayerInput,
    current_position: Vec3,
    camera_forward: Vec3,
    delta_time: f32,
) -> Vec3 {
    if input.forward == 0.0 && input.right == 0.0 && input.up == 0.0 {
        return current_position;
    }

    let forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    let right = forward.cross(Vec3::Y).normalize_or_zero();

    let mut velocity = forward * input.forward + right * input.right + Vec3::Y * input.up;
    velocity = velocity.normalize_or_zero();

    let speed = if input.sprint {
        MOVEMENT_SPEED * SPRINT_MULTIPLIER
    } else {
        MOVEMENT_SPEED
    };

    current_position + velocity * speed * delta_time
}

const MSG_CONNECT: u8 = 0x01;
const MSG_CONNECT_ACK: u8 = 0x02;
const MSG_PLAYER_JOINED: u8 = 0x03;
const MSG_PLAYER_LEFT: u8 = 0x04;
const MSG_HEARTBEAT: u8 = 0x05;
const MSG_DISCONNECT: u8 = 0x06;
const MSG_INPUT: u8 = 0x07;
const MSG_POSITION_UPDATE: u8 = 0x08;

#[derive(Debug, Clone)]
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
        x: f32,
        y: f32,
        z: f32,
        camera_forward: Vec3,
    },
}

impl Message {
    pub fn serialize(&self) -> io::Result<Vec<u8>> {
        let mut buf = Vec::new();

        match self {
            Message::Connect { name } => {
                buf.write_all(&[MSG_CONNECT])?;
                buf.write_all(&[name.len() as u8])?;
                buf.write_all(name.as_bytes())?;
            }
            Message::ConnectAck { client_id } => {
                buf.write_all(&[MSG_CONNECT_ACK])?;
                buf.write_all(&client_id.to_le_bytes())?;
            }
            Message::PlayerJoined { client_id, name } => {
                buf.write_all(&[MSG_PLAYER_JOINED])?;
                buf.write_all(&client_id.to_le_bytes())?;

                buf.write_all(&[name.len() as u8])?;
                buf.write_all(name.as_bytes())?;
            }
            Message::PlayerLeft { client_id, name } => {
                buf.write_all(&[MSG_PLAYER_LEFT])?;
                buf.write_all(&client_id.to_le_bytes())?;

                buf.write_all(&[name.len() as u8])?;
                buf.write_all(name.as_bytes())?;
            }
            Message::Heartbeat => {
                buf.write_all(&[MSG_HEARTBEAT])?;
            }
            Message::Disconnect => {
                buf.write_all(&[MSG_DISCONNECT])?;
            }
            Message::Input { input } => {
                buf.write_all(&[MSG_INPUT])?;
                buf.write_all(&input.forward.to_le_bytes())?;
                buf.write_all(&input.right.to_le_bytes())?;
                buf.write_all(&input.up.to_le_bytes())?;
                buf.write_all(&[input.sprint as u8])?;

                buf.write_all(&input.camera_forward.x.to_le_bytes())?;
                buf.write_all(&input.camera_forward.y.to_le_bytes())?;
                buf.write_all(&input.camera_forward.z.to_le_bytes())?;
            }
            Message::PositionUpdate {
                client_id,
                x,
                y,
                z,
                camera_forward,
            } => {
                buf.write_all(&[MSG_POSITION_UPDATE])?;
                buf.write_all(&client_id.to_le_bytes())?;
                buf.write_all(&x.to_le_bytes())?;
                buf.write_all(&y.to_le_bytes())?;
                buf.write_all(&z.to_le_bytes())?;
                buf.write_all(&camera_forward.x.to_le_bytes())?;
                buf.write_all(&camera_forward.y.to_le_bytes())?;
                buf.write_all(&camera_forward.z.to_le_bytes())?;
            }
        }

        Ok(buf)
    }

    pub fn deserialize(data: &[u8]) -> io::Result<Self> {
        if data.is_empty() {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "empty packet"));
        }

        let mut cursor = Cursor::new(data);
        let mut msg_type = [0u8; 1];
        cursor.read_exact(&mut msg_type)?;

        match msg_type[0] {
            MSG_CONNECT => {
                let mut len = [0u8; 1];
                cursor.read_exact(&mut len)?;

                let mut name_bytes = vec![0u8; len[0] as usize];
                cursor.read_exact(&mut name_bytes)?;

                let name = String::from_utf8(name_bytes)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf8"))?;

                Ok(Message::Connect { name })
            }
            MSG_CONNECT_ACK => {
                let mut id_bytes = [0u8; 4];
                cursor.read_exact(&mut id_bytes)?;

                Ok(Message::ConnectAck {
                    client_id: u32::from_le_bytes(id_bytes),
                })
            }
            MSG_PLAYER_JOINED => {
                let mut id_bytes = [0u8; 4];
                cursor.read_exact(&mut id_bytes)?;

                let client_id = u32::from_le_bytes(id_bytes);

                let mut len = [0u8; 1];
                cursor.read_exact(&mut len)?;

                let mut name_bytes = vec![0u8; len[0] as usize];
                cursor.read_exact(&mut name_bytes)?;

                let name = String::from_utf8(name_bytes)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf8"))?;

                Ok(Message::PlayerJoined { client_id, name })
            }
            MSG_PLAYER_LEFT => {
                let mut id_bytes = [0u8; 4];
                cursor.read_exact(&mut id_bytes)?;

                let client_id = u32::from_le_bytes(id_bytes);

                let mut len = [0u8; 1];
                cursor.read_exact(&mut len)?;

                let mut name_bytes = vec![0u8; len[0] as usize];
                cursor.read_exact(&mut name_bytes)?;

                let name = String::from_utf8(name_bytes)
                    .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid utf8"))?;

                Ok(Message::PlayerLeft { client_id, name })
            }
            MSG_HEARTBEAT => Ok(Message::Heartbeat),
            MSG_DISCONNECT => Ok(Message::Disconnect),
            MSG_INPUT => {
                let mut forward_bytes = [0u8; 4];
                cursor.read_exact(&mut forward_bytes)?;

                let mut right_bytes = [0u8; 4];
                cursor.read_exact(&mut right_bytes)?;

                let mut up_bytes = [0u8; 4];
                cursor.read_exact(&mut up_bytes)?;

                let mut sprint = [0u8; 1];
                cursor.read_exact(&mut sprint)?;

                let mut camera_x = [0u8; 4];
                cursor.read_exact(&mut camera_x)?;

                let mut camera_y = [0u8; 4];
                cursor.read_exact(&mut camera_y)?;

                let mut camera_z = [0u8; 4];
                cursor.read_exact(&mut camera_z)?;

                Ok(Message::Input {
                    input: PlayerInput {
                        forward: f32::from_le_bytes(forward_bytes),
                        right: f32::from_le_bytes(right_bytes),
                        up: f32::from_le_bytes(up_bytes),
                        sprint: sprint[0] != 0,
                        camera_forward: Vec3::new(
                            f32::from_le_bytes(camera_x),
                            f32::from_le_bytes(camera_y),
                            f32::from_le_bytes(camera_z),
                        ),
                    },
                })
            }
            MSG_POSITION_UPDATE => {
                let mut id_bytes = [0u8; 4];
                cursor.read_exact(&mut id_bytes)?;

                let mut x_bytes = [0u8; 4];
                cursor.read_exact(&mut x_bytes)?;

                let mut y_bytes = [0u8; 4];
                cursor.read_exact(&mut y_bytes)?;

                let mut z_bytes = [0u8; 4];
                cursor.read_exact(&mut z_bytes)?;

                let mut camera_x = [0u8; 4];
                cursor.read_exact(&mut camera_x)?;

                let mut camera_y = [0u8; 4];
                cursor.read_exact(&mut camera_y)?;

                let mut camera_z = [0u8; 4];
                cursor.read_exact(&mut camera_z)?;

                Ok(Message::PositionUpdate {
                    client_id: u32::from_le_bytes(id_bytes),
                    x: f32::from_le_bytes(x_bytes),
                    y: f32::from_le_bytes(y_bytes),
                    z: f32::from_le_bytes(z_bytes),
                    camera_forward: Vec3::new(
                        f32::from_le_bytes(camera_x),
                        f32::from_le_bytes(camera_y),
                        f32::from_le_bytes(camera_z),
                    ),
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unknown message type",
            )),
        }
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
