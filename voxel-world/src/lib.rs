pub mod bridge;
pub mod command;
pub mod envelope;
pub mod event;
pub mod physics;
pub mod player;
pub mod request;
mod terrain;

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
    physics::Physics,
    player::{PlayerInput, PlayerState},
    request::{PendingRequest, Pong},
    terrain::{
        CHUNK_RENDER_DISTANCE, Terrain, chunk_in_range, chunks_in_radius, world_to_chunk_pos,
    },
};

pub const MOVEMENT_SPEED: f32 = 10.0;
pub const SPRINT_MULTIPLIER: f32 = 2.0;
pub const THRUST: f32 = 20.0;

pub struct VoxelWorld {
    players: HashMap<u32, PlayerState>,
    terrain: Terrain,
    physics: Physics,
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
            physics: Physics::init(),
            events: Vec::new(),
            next_id: 1,
        }
    }

    pub fn run(
        mut self,
        mut command_rx: UnboundedReceiver<Envelope<WorldCommand>>,
        mut req_rx: UnboundedReceiver<PendingRequest>,
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
        req_rx: &mut UnboundedReceiver<PendingRequest>,
        dt: f32,
    ) -> Vec<Envelope<WorldEvent>> {
        while let Ok(cmd) = command_rx.try_recv() {
            self.execute(cmd);
        }

        while let Ok(req) = req_rx.try_recv() {
            self.handle(req);
        }

        // movement
        self.process_player_inputs();
        self.physics.step(dt);
        self.broadcast_movement();

        // terrain
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

    fn handle(&mut self, req: PendingRequest) {
        match req {
            PendingRequest::Connect(call) => {
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
            PendingRequest::Ping(call) => {
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
        let body = self.physics.add_body(Vec3::new(0.0, 60.0, 0.0));
        self.players.insert(id, PlayerState::new(name, body));

        id
    }

    fn remove_player(&mut self, id: u32) {
        if let Some(player) = self.players.remove(&id) {
            self.physics.remove_body(player.body);

            let event = Envelope::broadcast(PlayerLeft {
                id,
                name: player.name,
            });

            self.events.push(event);
        }
    }

    fn sync_player_chunks(&mut self) {
        for (&player_id, player_state) in self.players.iter_mut() {
            let chunk_pos = world_to_chunk_pos(self.physics.position(player_state.body));

            if player_state.chunks.anchor.replace(chunk_pos) == Some(chunk_pos) {
                continue;
            }

            // unload
            let to_unload: Vec<_> = player_state
                .chunks
                .loaded
                .iter()
                .copied()
                .filter(|&pos| !chunk_in_range(chunk_pos, pos, CHUNK_RENDER_DISTANCE))
                .collect();

            for pos in to_unload {
                player_state.chunks.loaded.remove(&pos);
                self.events.push(Envelope::to(
                    player_id,
                    ChunkUnloaded {
                        pos: pos.to_array(),
                    },
                ));
            }

            // load
            let mut needed: Vec<_> = chunks_in_radius(chunk_pos, CHUNK_RENDER_DISTANCE)
                .into_iter()
                .filter(|pos| !player_state.chunks.loaded.contains(pos))
                .collect();
            needed.sort_by_key(|pos| pos.distance_squared(chunk_pos));

            for pos in needed {
                if let Some(data) = self.terrain.get(pos) {
                    player_state.chunks.loaded.insert(pos);
                    self.events.push(Envelope::to(
                        player_id,
                        ChunkLoaded {
                            pos: pos.to_array(),
                            data,
                        },
                    ));
                } else {
                    self.terrain.request(pos);
                }
            }
        }
    }

    fn poll_terrain(&mut self) {
        for (pos, data) in self.terrain.poll() {
            for (&player_id, player) in &mut self.players {
                if player.chunks.needs(pos) && !player.chunks.loaded.contains(&pos) {
                    player.chunks.loaded.insert(pos);
                    let event = Envelope::to(
                        player_id,
                        ChunkLoaded {
                            pos: pos.to_array(),
                            data: data.clone(),
                        },
                    );
                    self.events.push(event);
                }
            }
        }
    }

    fn process_player_inputs(&mut self) {
        for player_state in self.players.values() {
            let PlayerInput { dir, look, sprint } = player_state.input;
            let dir = Vec3::from_array(dir);
            let look = Vec3::from_array(look);

            let forward = Vec3::new(look.x, 0.0, look.z).normalize_or_zero();
            let right = forward.cross(Vec3::Y);
            let move_dir = forward * dir.x + right * dir.z + Vec3::Y * dir.y;

            let speed_mult = if sprint { SPRINT_MULTIPLIER } else { 1.0 };
            let force = move_dir * THRUST * speed_mult;

            self.physics.set_force(player_state.body, force);
        }
    }

    fn broadcast_movement(&mut self) {
        for (player_id, player_state) in &self.players {
            let event = Envelope::broadcast(PlayerMoved {
                id: *player_id,
                pos: self.physics.position(player_state.body).to_array(),
                look: player_state.input.look,
            });
            self.events.push(event);
        }
    }
}
