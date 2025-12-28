use crate::{
    Systems,
    network::{
        Connection,
        events::{Connected, PlayerJoined, PlayerLeft, PositionUpdate},
    },
};
use bevy::prelude::*;
use std::collections::HashMap;
use voxel_net::message::ClientMessage;
use voxel_world::player::PlayerInput;

#[allow(dead_code)]
#[derive(Component)]
pub struct LocalPlayer {
    pub id: Option<u32>,
    pub name: String,
    pub input: PlayerInput,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct RemotePlayer {
    pub id: u32,
    pub name: String,
}

#[derive(Default, Resource)]
pub struct PlayerEntities(pub HashMap<u32, Entity>);

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntities>()
            .add_observer(on_connected)
            .add_observer(on_player_joined)
            .add_observer(on_player_left)
            .add_observer(on_position_update)
            .add_systems(
                Update,
                (read_input, send_input)
                    .chain()
                    .in_set(Systems::Input)
                    .run_if(has_local_player),
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
            id: Some(event.id),
            name: event.name.clone(),
            input: PlayerInput::default(),
        },
        Transform::from_xyz(0.0, 60.0, 0.0),
    ));
}

fn on_player_joined(
    on: On<PlayerJoined>,
    local_player: Single<&LocalPlayer>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut players: ResMut<PlayerEntities>,
) {
    let event = on.event();
    if let Some(id) = local_player.id
        && id == event.id
    {
        return;
    }

    println!("{} joined", event.name);

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
        ))
        .id();

    players.0.insert(event.id, entity);
}

fn on_player_left(on: On<PlayerLeft>, mut commands: Commands, mut players: ResMut<PlayerEntities>) {
    let event = on.event();
    println!("{} left", event.name);

    if let Some(entity) = players.0.remove(&event.id) {
        commands.entity(entity).despawn();
    }
}

fn on_position_update(
    on: On<PositionUpdate>,
    mut commands: Commands,
    player: Single<(&LocalPlayer, &mut Transform)>,
    player_entities: ResMut<PlayerEntities>,
) {
    let (local_player, mut transform) = player.into_inner();
    let event = on.event();

    if let Some(id) = local_player.id
        && id == event.id
    {
        transform.translation = event.pos;
        return;
    }

    if let Some(&entity) = player_entities.0.get(&event.id)
        && let Ok(mut entity_commands) = commands.get_entity(entity)
    {
        entity_commands.insert(Transform::from_translation(event.pos));
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

    local_player.input.dir = input_dir;
    local_player.input.sprint = sprint;
}

pub fn send_input(connection: Res<Connection>, local_player: Single<&LocalPlayer>) {
    connection.send(ClientMessage::Input {
        input: local_player.input.clone(),
    });
}
