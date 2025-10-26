use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;
use shared::PlayerInput;

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
    mut mouse_motion: MessageReader<MouseMotion>,
    camera: Single<(&mut Camera, &mut Transform)>,
    mut player_input: Single<&mut PlayerInput>,
    time: Res<Time>,
) {
    let (mut camera, mut transform) = camera.into_inner();

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

    player_input.camera_forward = transform.forward().into();
}

fn follow_player(
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    player_transform: Single<&Transform, (With<LocalPlayer>, Without<Camera>)>,
) {
    camera_transform.translation = player_transform.translation;
}
