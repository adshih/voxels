use glam::{IVec3, Vec3};
use std::collections::HashMap;

use crate::{
    commands::WorldCommand,
    events::WorldEvent,
    player::{PlayerInput, PlayerState},
    terrain::{CHUNK_RENDER_DISTANCE, Terrain, chunks_in_radius, world_to_chunk_pos},
};

pub mod commands;
pub mod events;
pub mod player;
mod terrain;

pub struct VoxelWorld {
    players: HashMap<u32, PlayerState>,
    terrain: Terrain,
    events: Vec<WorldEvent>,
    next_id: u32,
}

impl VoxelWorld {
    const MOVEMENT_SPEED: f32 = 10.0;
    const SPRINT_MULTIPLIER: f32 = 2.0;

    pub fn new(seed: u32) -> Self {
        Self {
            players: HashMap::new(),
            terrain: Terrain::new(seed),
            events: Vec::new(),
            next_id: 1,
        }
    }

    pub fn add_player(&mut self) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.players.insert(id, PlayerState::default());

        id
    }

    pub fn remove_player(&mut self, id: u32) {
        self.players.remove(&id);
    }

    pub fn tick(&mut self, dt: f32) -> Vec<WorldEvent> {
        self.process_player_inputs(dt);
        self.sync_player_chunks();
        self.poll_terrain();

        std::mem::take(&mut self.events)
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

    fn sync_player_chunks(&mut self) {
        for player in &mut self.players.values_mut() {
            let chunk_pos = world_to_chunk_pos(player.pos);

            if player.chunk_anchor != Some(chunk_pos) {
                player.chunk_anchor = Some(chunk_pos);

                for pos in chunks_in_radius(chunk_pos, CHUNK_RENDER_DISTANCE) {
                    self.terrain.request(pos);
                }
            }
        }
    }

    fn poll_terrain(&mut self) {
        for (pos, data) in self.terrain.poll() {
            for (&player_id, player) in &self.players {
                if player_needs_chunk(player, pos) {
                    self.events.push(WorldEvent::ChunkLoaded {
                        for_player: player_id,
                        pos,
                        data: data.clone(),
                    });
                }
            }
        }
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

fn player_needs_chunk(player: &PlayerState, chunk_pos: IVec3) -> bool {
    match player.chunk_anchor {
        Some(anchor) => {
            let diff = chunk_pos - anchor;
            diff.x.abs() <= CHUNK_RENDER_DISTANCE
                && diff.y.abs() <= CHUNK_RENDER_DISTANCE
                && diff.z.abs() <= CHUNK_RENDER_DISTANCE
        }
        None => false,
    }
}
