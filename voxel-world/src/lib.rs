use glam::{IVec3, Vec3};
use std::collections::HashMap;
use voxel_core::VoxelBuffer;

use crate::{commands::WorldCommand, events::WorldEvent};

pub mod commands;
pub mod events;

#[derive(Default, Debug, Clone)]
pub struct PlayerInput {
    pub dir: Vec3,
    pub look: Vec3,
    pub sprint: bool,
}

pub struct PlayerState {
    pub pos: Vec3,
    pub look: Vec3,
    pub input: PlayerInput,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            pos: Vec3::new(0.0, 60.0, 0.0),
            look: Vec3::default(),
            input: PlayerInput::default(),
        }
    }
}

pub struct VoxelWorld {
    players: HashMap<u32, PlayerState>,
    next_id: u32,
    tick: u64,
    events: Vec<WorldEvent>,
    _chunks: HashMap<IVec3, VoxelBuffer>,
    _seed: u64,
}

impl VoxelWorld {
    const MOVEMENT_SPEED: f32 = 10.0;
    const SPRINT_MULTIPLIER: f32 = 2.0;

    pub fn new(seed: u64) -> Self {
        Self {
            players: HashMap::new(),
            tick: 0,
            next_id: 1,
            events: Vec::new(),
            _chunks: HashMap::new(),
            _seed: seed,
        }
    }

    pub fn players(&self) -> &HashMap<u32, PlayerState> {
        &self.players
    }

    pub fn add_player(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.players.insert(id, PlayerState::default());

        id
    }

    pub fn tick(&mut self, dt: f32) {
        self.process_player_inputs(dt);
        self.tick += 1;
    }

    pub fn execute(&mut self, cmd: WorldCommand) {
        match cmd {
            WorldCommand::PlayerMove { id, input } => {
                if let Some(player) = self.players.get_mut(&id) {
                    player.input = input;
                }
            }
        }
    }

    pub fn drain_events(&mut self) -> Vec<WorldEvent> {
        std::mem::take(&mut self.events)
    }

    fn process_player_inputs(&mut self, dt: f32) {
        let mut events = Vec::new();

        for (player_id, player_state) in &mut self.players {
            let PlayerInput { dir, look, sprint } = player_state.input;

            player_state.look = look;

            if dir == Vec3::ZERO {
                continue;
            }

            let forward = Vec3::new(look.x, 0.0, look.z).normalize_or_zero();
            let right = forward.cross(Vec3::Y);
            let move_dir = forward * dir.x + right * dir.z + Vec3::Y * dir.y;

            let speed_mult = if sprint { Self::SPRINT_MULTIPLIER } else { 1.0 };
            let speed = Self::MOVEMENT_SPEED * speed_mult;

            player_state.pos += move_dir * speed * dt;

            events.push(WorldEvent::PlayerMoved {
                id: *player_id,
                pos: player_state.pos,
                look: player_state.look,
            });
        }

        self.events.extend(events);
    }
}
