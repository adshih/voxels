use bevy::prelude::*;
use voxel_world::PlayerInput;

use crate::Systems;

#[allow(dead_code)]
#[derive(Component)]
pub struct LocalPlayer {
    pub name: String,
    pub input: PlayerInput,
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
            input: PlayerInput::default(),
        },
        Transform::from_xyz(0.0, 60.0, 0.0),
    ));
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
