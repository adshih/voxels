use bevy::prelude::*;
use shared::{PlayerInput, calculate_movement};

use crate::{Systems, network::client::NetworkClient};

#[allow(dead_code)]
#[derive(Component)]
pub struct LocalPlayer {
    pub name: String,
}

#[allow(dead_code)]
#[derive(Component)]
pub struct RemotePlayer {
    pub id: u32,
    pub name: String,
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                read_input.in_set(Systems::Input),
                move_player
                    .in_set(Systems::Movement)
                    .run_if(not(resource_exists::<NetworkClient>)),
            ),
        );
    }
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        LocalPlayer {
            name: "LocalPlayer".to_string(),
        },
        PlayerInput::default(),
        Transform::from_xyz(0.0, 60.0, 0.0),
    ));
}

fn read_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut input: Single<&mut PlayerInput, With<LocalPlayer>>,
) {
    input.forward = 0.0;
    input.right = 0.0;
    input.up = 0.0;

    if keyboard.pressed(KeyCode::KeyW) {
        input.forward += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input.forward -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input.right += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyA) {
        input.right -= 1.0;
    }
    if keyboard.pressed(KeyCode::Space) {
        input.up += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyC) {
        input.up -= 1.0;
    }

    input.sprint = keyboard.pressed(KeyCode::ShiftLeft);
}

fn move_player(
    player: Single<(&mut Transform, &PlayerInput), (With<LocalPlayer>, Without<Camera>)>,
    camera_transform: Single<&Transform, (With<Camera>, Without<LocalPlayer>)>,
    time: Res<Time>,
) {
    let (mut transform, input) = player.into_inner();

    let new_position = calculate_movement(
        input,
        transform.translation,
        camera_transform.forward().into(),
        time.delta_secs(),
    );

    transform.translation = new_position;
}
