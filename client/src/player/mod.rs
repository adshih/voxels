mod camera;

use std::collections::HashMap;

use bevy::prelude::*;
use voxel_world::{command::MovePlayer, event::*, player::PlayerInput};

use crate::{
    connection::bridge::{FromWorld, WorldBridge},
    is_cursor_locked,
    player::camera::*,
};

#[derive(Component)]
pub struct Head(pub Entity);

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
            .add_observer(on_player_joined)
            .add_observer(on_player_left)
            .add_observer(on_position_update)
            .add_observer(on_connected)
            .add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                (
                    camera_look.run_if(is_cursor_locked),
                    read_input,
                    send_input,
                    follow_player,
                )
                    .chain(),
            );
    }
}

fn on_connected(
    on: On<Connected>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let event = on.event();
    let root = spawn_player_model(
        &mut commands,
        &mut meshes,
        &mut materials,
        Color::srgb(0.3, 0.5, 0.9),
    );

    commands.entity(root).insert(LocalPlayer {
        id: event.id,
        name: event.name.clone(),
        input: PlayerInput::default(),
    });
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

    let entity = spawn_player_model(&mut commands, &mut meshes, &mut materials, Color::WHITE);
    commands.entity(entity).insert((
        Name::new(format!("RemotePlayer_{}", event.id)),
        RemotePlayer {
            id: event.id,
            name: event.name.clone(),
        },
    ));

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
    heads: Query<&Head>,
    mut transforms: Query<&mut Transform>,
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

    let look = Vec3::from_array(event.look).normalize_or_zero();

    if let Ok(mut transform) = transforms.get_mut(entity) {
        transform.translation = Vec3::from_array(event.pos);

        let flat = look.with_y(0.0);
        if flat != Vec3::ZERO {
            transform.look_to(flat, Vec3::Y);
        }
    }

    let Ok(&Head(head)) = heads.get(entity) else {
        return;
    };
    if let Ok(mut head_transform) = transforms.get_mut(head) {
        head_transform.rotation = Quat::from_rotation_x(look.y.asin());
    }
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

fn spawn_player_model(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    color: Color,
) -> Entity {
    let material = materials.add(color);

    let root = commands
        .spawn((Transform::from_xyz(0.0, 60.0, 0.0), Visibility::default()))
        .id();

    commands.spawn((
        Mesh3d(meshes.add(Capsule3d::default())),
        MeshMaterial3d(material.clone()),
        ChildOf(root),
    ));

    let head = commands
        .spawn((
            Mesh3d(meshes.add(Cuboid::new(0.7, 0.7, 0.7))),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, 1.0, 0.0),
            ChildOf(root),
        ))
        .id();

    commands.entity(root).insert(Head(head));
    root
}
