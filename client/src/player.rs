use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use voxel_world::{TICK_RATE, command::MovePlayer, event::*, player::PlayerInput};

use crate::{
    Systems,
    connection::bridge::{FromWorld, WorldBridge},
};

const INTERP_DELAY: f64 = 2.0;
const MAX_DRIFT_TICKS: f64 = 8.0;
const RESYNC_RATE: f64 = 2.0;

#[derive(Clone, Copy)]
struct Snapshot {
    tick: u64,
    pos: Vec3,
    look: Vec3,
}

#[derive(Component, Default)]
pub struct SnapshotBuffer {
    snapshots: VecDeque<Snapshot>,
}

impl SnapshotBuffer {
    fn push(&mut self, snap: Snapshot) {
        if self.snapshots.back().is_some_and(|b| snap.tick <= b.tick) {
            return;
        }
        self.snapshots.push_back(snap);
        while self.snapshots.len() > 32 {
            self.snapshots.pop_front();
        }
    }

    fn sample(&self, t: f64) -> Option<(Snapshot, Snapshot, f32)> {
        let s = &self.snapshots;
        for i in 0..s.len().saturating_sub(1) {
            let (a, b) = (s[i], s[i + 1]);
            if (a.tick as f64) <= t && t <= (b.tick as f64) {
                let span = (b.tick - a.tick) as f64;
                let alpha = if span > 0.0 {
                    (t - a.tick as f64) / span
                } else {
                    0.0
                };
                return Some((a, b, alpha as f32));
            }
        }
        None
    }
}

#[derive(Resource, Default)]
pub struct RenderClock {
    tick: f64,
    latest_tick: u64,
    initialized: bool,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct LocalPlayer {
    pub id: u32,
    pub name: String,
    pub input: PlayerInput,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct RemotePlayer {
    pub id: u32,
    pub name: String,
}

#[derive(Event)]
pub struct Connected {
    pub id: u32,
    pub name: String,
}

#[derive(Default, Resource)]
pub struct PlayerEntities(pub HashMap<u32, Entity>);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntities>()
            .init_resource::<RenderClock>()
            .add_observer(on_player_joined)
            .add_observer(on_player_left)
            .add_observer(on_position_update)
            .add_observer(on_connected)
            .add_systems(
                Update,
                (read_input, send_input)
                    .chain()
                    .in_set(Systems::Input)
                    .run_if(has_local_player),
            )
            .add_systems(
                Update,
                (advance_render_clock, interpolate_players)
                    .chain()
                    .in_set(Systems::Movement),
            );
    }
}

fn has_local_player(player: Option<Single<&LocalPlayer>>) -> bool {
    player.is_some()
}

fn on_connected(on: On<Connected>, mut commands: Commands) {
    let event = on.event();
    commands.spawn((
        LocalPlayer {
            id: event.id,
            name: event.name.clone(),
            input: PlayerInput::default(),
        },
        Transform::from_xyz(0.0, 60.0, 0.0),
        SnapshotBuffer::default(),
    ));
}

fn on_player_joined(
    on: On<FromWorld<PlayerJoined>>,
    local_player: Single<&LocalPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: ResMut<PlayerEntities>,
) {
    let event = on.event();

    println!("{} joined", event.name);

    if local_player.id == event.id {
        return;
    }

    let entity = commands
        .spawn((
            Name::new(format!("RemotePlayer_{}", event.id)),
            RemotePlayer {
                id: event.id,
                name: event.name.clone(),
            },
            Transform::from_xyz(0.0, 60.0, 0.0),
            Mesh3d(meshes.add(Capsule3d::default())),
            MeshMaterial3d(materials.add(Color::WHITE)),
            SnapshotBuffer::default(),
        ))
        .id();

    players.0.insert(event.id, entity);
}

fn on_player_left(
    on: On<FromWorld<PlayerLeft>>,
    mut commands: Commands,
    mut players: ResMut<PlayerEntities>,
) {
    let event = on.event();
    println!("{} left", event.name);

    if let Some(entity) = players.0.remove(&event.id) {
        commands.entity(entity).despawn();
    }
}

fn on_position_update(
    on: On<FromWorld<PlayerMoved>>,
    local: Single<(Entity, &LocalPlayer)>,
    remotes: Res<PlayerEntities>,
    mut buffers: Query<&mut SnapshotBuffer>,
    mut clock: ResMut<RenderClock>,
) {
    let event = on.event();
    let (local_entity, local_player) = local.into_inner();

    let entity = if event.id == local_player.id {
        local_entity
    } else if let Some(&e) = remotes.0.get(&event.id) {
        e
    } else {
        return;
    };

    if let Ok(mut buffer) = buffers.get_mut(entity) {
        buffer.push(Snapshot {
            tick: event.tick,
            pos: Vec3::from_array(event.pos),
            look: Vec3::from_array(event.look),
        });
    }

    clock.latest_tick = clock.latest_tick.max(event.tick);
}

fn read_input(keyboard: Res<ButtonInput<KeyCode>>, mut local_player: Single<&mut LocalPlayer>) {
    let mut input_dir = Vec3::ZERO;
    let sprint = keyboard.pressed(KeyCode::ShiftLeft);

    if keyboard.pressed(KeyCode::KeyW) {
        input_dir.x += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input_dir.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input_dir.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input_dir.z -= 1.0;
    }
    if keyboard.pressed(KeyCode::Space) {
        input_dir.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyC) {
        input_dir.y -= 1.0;
    }

    local_player.input.dir = input_dir.to_array();
    local_player.input.sprint = sprint;
}

pub fn send_input(world: Res<WorldBridge>, local_player: Single<&LocalPlayer>) {
    world.send(MovePlayer {
        input: local_player.input.clone(),
    });
}

fn advance_render_clock(time: Res<Time>, mut clock: ResMut<RenderClock>) {
    if clock.latest_tick == 0 {
        return;
    }
    let target = clock.latest_tick as f64 - INTERP_DELAY;

    if !clock.initialized {
        clock.tick = target;
        clock.initialized = true;
        return;
    }

    clock.tick += time.delta_secs_f64() * TICK_RATE as f64;

    let err = target - clock.tick;
    if err.abs() > MAX_DRIFT_TICKS {
        clock.tick = target;
    } else {
        let k = 1.0 - (-RESYNC_RATE * time.delta_secs_f64()).exp();
        clock.tick += err * k;
    }
}

fn interpolate_players(
    clock: Res<RenderClock>,
    mut query: Query<(&SnapshotBuffer, &mut Transform)>,
) {
    for (buffer, mut transform) in &mut query {
        if let Some((a, b, alpha)) = buffer.sample(clock.tick) {
            transform.translation = a.pos.lerp(b.pos, alpha);
        }
    }
}
