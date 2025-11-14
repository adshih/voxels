use bevy::prelude::*;
use shared::PlayerInput;

use crate::Systems;

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
        app.add_systems(Startup, spawn_player)
            .add_systems(Update, read_input.in_set(Systems::Input));
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
