pub mod bridge;
pub mod command;
pub mod envelope;
pub mod event;
pub mod player;
pub mod request;
mod terrain;

pub use bridge::Bridge;

use std::{
    collections::HashMap,
    time::{Duration, Instant},
};

use glam::Vec3;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    command::*,
    envelope::Envelope,
    event::*,
    player::{PlayerInput, PlayerState},
    request::{Pong, WorldRequest},
    terrain::{CHUNK_RENDER_DISTANCE, Terrain, chunk_in_range, chunks_in_radius, world_to_chunk_pos},
};

pub const MOVEMENT_SPEED: f32 = 10.0;
pub const SPRINT_MULTIPLIER: f32 = 2.0;

pub struct VoxelWorld {
    players: HashMap<u32, PlayerState>,
    terrain: Terrain,
    events: Vec<Envelope<WorldEvent>>,
    next_id: u32,
}

impl VoxelWorld {
    const TICK_RATE: f32 = 60.0;
    const DT: f32 = 1.0 / Self::TICK_RATE;

    pub fn new(seed: u32) -> Self {
        Self {
            players: HashMap::new(),
            terrain: Terrain::new(seed),
            events: Vec::new(),
            next_id: 1,
        }
    }

    pub fn run(
        mut self,
        mut command_rx: UnboundedReceiver<Envelope<WorldCommand>>,
        mut req_rx: UnboundedReceiver<WorldRequest>,
        event_tx: UnboundedSender<Envelope<WorldEvent>>,
    ) {
        let mut next_tick = Instant::now();

        loop {
            for event in self.tick(&mut command_rx, &mut req_rx, Self::DT) {
                let _ = event_tx.send(event);
            }

            next_tick += Duration::from_secs_f32(Self::DT);
            std::thread::sleep(next_tick.saturating_duration_since(Instant::now()));
        }
    }

    fn tick(
        &mut self,
        command_rx: &mut UnboundedReceiver<Envelope<WorldCommand>>,
        req_rx: &mut UnboundedReceiver<WorldRequest>,
        dt: f32,
    ) -> Vec<Envelope<WorldEvent>> {
        while let Ok(cmd) = command_rx.try_recv() {
            self.execute(cmd);
        }

        while let Ok(req) = req_rx.try_recv() {
            self.handle(req);
        }

        self.process_player_inputs(dt);
        self.sync_player_chunks();
        self.poll_terrain();

        std::mem::take(&mut self.events)
    }

    fn execute(&mut self, cmd: Envelope<WorldCommand>) {
        let id = cmd.from.unwrap();

        match cmd.payload {
            WorldCommand::Disconnect => self.remove_player(id),
            WorldCommand::MovePlayer(cmd) => {
                if let Some(player) = self.players.get_mut(&id) {
                    player.input = cmd.input;
                }
            }
        }
    }

    fn handle(&mut self, req: WorldRequest) {
        match req {
            WorldRequest::Connect(call) => {
                let id = self.add_player(call.payload.name.clone());
                call.reply(id);

                for (pid, state) in &self.players {
                    if *pid == id {
                        continue;
                    }

                    self.events.push(Envelope::to(
                        id,
                        PlayerJoined {
                            id: *pid,
                            name: state.name.clone(),
                        },
                    ));
                }
            }
            WorldRequest::Ping(call) => {
                call.reply(Pong);
            }
        }
    }

    fn add_player(&mut self, name: String) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.events.push(Envelope::broadcast(PlayerJoined {
            id,
            name: name.clone(),
        }));
        self.players.insert(id, PlayerState::new(name));

        id
    }

    fn remove_player(&mut self, id: u32) {
        if let Some(player) = self.players.remove(&id) {
            let event = Envelope::broadcast(PlayerLeft {
                id,
                name: player.name,
            });

            self.events.push(event);
        }
    }

    fn sync_player_chunks(&mut self) {
        for (&player_id, player_state) in self.players.iter_mut() {
            let chunk_pos = world_to_chunk_pos(player_state.pos);

            if player_state.chunk_anchor.replace(chunk_pos) == Some(chunk_pos) {
                continue;
            }

            // unload
            let to_unload: Vec<_> = player_state
                .loaded_chunks
                .iter()
                .copied()
                .filter(|&pos| !chunk_in_range(chunk_pos, pos, CHUNK_RENDER_DISTANCE))
                .collect();

            for pos in to_unload {
                player_state.loaded_chunks.remove(&pos);
                self.events
                    .push(Envelope::to(player_id, ChunkUnloaded { pos }));
            }

            // load
            let mut needed: Vec<_> = chunks_in_radius(chunk_pos, CHUNK_RENDER_DISTANCE)
                .into_iter()
                .filter(|pos| !player_state.loaded_chunks.contains(pos))
                .collect();
            needed.sort_by_key(|pos| pos.distance_squared(chunk_pos));

            for pos in needed {
                if let Some(data) = self.terrain.get(pos) {
                    player_state.loaded_chunks.insert(pos);
                    self.events
                        .push(Envelope::to(player_id, ChunkLoaded { pos, data }));
                } else {
                    self.terrain.request(pos);
                }
            }
        }
    }

    fn poll_terrain(&mut self) {
        for (pos, data) in self.terrain.poll() {
            for (&player_id, player) in &mut self.players {
                if player.needs_chunk(pos) && !player.loaded_chunks.contains(&pos) {
                    player.loaded_chunks.insert(pos);
                    let event = Envelope::to(
                        player_id,
                        ChunkLoaded {
                            pos,
                            data: data.clone(),
                        },
                    );
                    self.events.push(event);
                }
            }
        }
    }

    fn process_player_inputs(&mut self, dt: f32) {
        for (player_id, player_state) in &mut self.players {
            let PlayerInput { dir, look, sprint } = player_state.input;

            player_state.look = look;

            if dir == Vec3::ZERO {
                continue;
            }

            let forward = Vec3::new(look.x, 0.0, look.z).normalize_or_zero();
            let right = forward.cross(Vec3::Y);
            let move_dir = forward * dir.x + right * dir.z + Vec3::Y * dir.y;

            let speed_mult = if sprint { SPRINT_MULTIPLIER } else { 1.0 };
            let speed = MOVEMENT_SPEED * speed_mult;

            player_state.pos += move_dir * speed * dt;

            let event = Envelope::broadcast(PlayerMoved {
                id: *player_id,
                pos: player_state.pos,
                look: player_state.look,
            });
            self.events.push(event);
        }
    }
}
