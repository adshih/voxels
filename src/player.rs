use bevy::prelude::*;

use crate::Systems;

#[derive(Component)]
pub struct Player {
    movement_speed: f32,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            movement_speed: 10.0,
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_player).add_systems(
            Update,
            (
                read_input.in_set(Systems::Input),
                move_player.in_set(Systems::Movement),
            ),
        );
    }
}

#[derive(Component, Default, Debug)]
pub struct Input {
    movement: Vec3,
    sprint: bool,
}

fn spawn_player(mut commands: Commands) {
    commands.spawn((
        Name::new("Player"),
        Player::default(),
        Input::default(),
        Transform::from_xyz(0.0, 30.0, 0.0),
    ));
}

fn read_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<&mut Input, With<Player>>,
) {
    let mut input = player_query.single_mut().expect("Could not find player");

    input.movement = Vec3::ZERO;

    if keyboard.pressed(KeyCode::KeyW) {
        input.movement.z += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyS) {
        input.movement.z -= 1.0;
    }

    if keyboard.pressed(KeyCode::KeyA) {
        input.movement.x -= 1.0;
    }
    if keyboard.pressed(KeyCode::KeyD) {
        input.movement.x += 1.0;
    }

    if keyboard.pressed(KeyCode::Space) {
        input.movement.y += 1.0;
    }
    if keyboard.pressed(KeyCode::KeyC) {
        input.movement.y -= 1.0;
    }

    input.sprint = keyboard.pressed(KeyCode::ShiftLeft);
}

fn move_player(
    mut player_query: Query<(&Player, &mut Transform, &Input), Without<Camera>>,
    camera_query: Query<&Transform, (With<Camera>, Without<Player>)>,
    time: Res<Time>,
) {
    let (player, mut transform, input) = player_query.single_mut().expect("Could not find player");
    let camera_transform = camera_query.single().expect("Could not find camera");

    if input.movement == Vec3::ZERO {
        return;
    }

    let camera_forward = camera_transform.forward();
    let camera_right = camera_transform.right();

    let forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize();
    let right = Vec3::new(camera_right.x, 0.0, camera_right.z).normalize();

    let mut velocity =
        forward * input.movement.z + right * input.movement.x + Vec3::Y * input.movement.y;

    let speed = if input.sprint {
        player.movement_speed * 2.0
    } else {
        player.movement_speed
    };

    velocity = velocity.normalize_or_zero();
    transform.translation += velocity * speed * time.delta_secs();
}
