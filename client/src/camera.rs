use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use crate::Systems;
use crate::player::LocalPlayer;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            Update,
            (
                camera_look.in_set(Systems::Input),
                follow_player.in_set(Systems::PostMovement),
            ),
        );
    }
}

#[derive(Component)]
struct Camera {
    sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            sensitivity: 2.0,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

fn spawn_camera(mut commands: Commands) {
    let camera = Camera::default();

    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::default(),
        camera,
    ));
}

fn camera_look(
    mut mouse_motion: EventReader<MouseMotion>,
    mut camera_query: Query<(&mut Camera, &mut Transform)>,
    time: Res<Time>,
) {
    let (mut camera, mut transform) = camera_query.single_mut().expect("Could not find camera");

    for event in mouse_motion.read() {
        camera.yaw -= event.delta.x * camera.sensitivity * time.delta_secs();
        camera.pitch -= event.delta.y * camera.sensitivity * time.delta_secs();
    }

    camera.pitch = camera.pitch.clamp(-89.9, 89.9);

    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        camera.yaw.to_radians(),
        camera.pitch.to_radians(),
        0.0,
    );
}

fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera>>,
    player_query: Query<&Transform, (With<LocalPlayer>, Without<Camera>)>,
) {
    let mut camera_transform = camera_query.single_mut().expect("Could not find camera");
    let player_transform = player_query.single().expect("Could not find player");

    camera_transform.translation = player_transform.translation;
}
