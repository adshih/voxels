use bevy::{input::mouse::MouseMotion, prelude::*};

use crate::player::LocalPlayer;

const CAMERA_DISTANCE: f32 = 8.0;
const CAMERA_HEIGHT: f32 = 2.0;

#[derive(Component)]
pub struct Camera {
    sensitivity: f32,
    pitch: f32,
    yaw: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            sensitivity: 0.1,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

pub fn spawn_camera(mut commands: Commands) {
    let camera = Camera::default();

    commands.spawn((
        Name::new("Camera"),
        Camera3d::default(),
        Transform::default(),
        camera,
    ));
}

pub fn camera_look(
    mut mouse_motion: MessageReader<MouseMotion>,
    camera: Single<(&mut Camera, &mut Transform)>,
    mut local_player: Single<&mut LocalPlayer>,
) {
    let (mut camera, mut transform) = camera.into_inner();

    for event in mouse_motion.read() {
        camera.yaw -= event.delta.x * camera.sensitivity;
        camera.pitch -= event.delta.y * camera.sensitivity;
    }

    camera.pitch = camera.pitch.clamp(-89.9, 89.9);

    transform.rotation = Quat::from_euler(
        EulerRot::YXZ,
        camera.yaw.to_radians(),
        camera.pitch.to_radians(),
        0.0,
    );

    local_player.input.look = Vec3::from(transform.forward()).to_array();
}

pub fn follow_player(
    mut camera_transform: Single<&mut Transform, With<Camera>>,
    player_transform: Single<&Transform, (With<LocalPlayer>, Without<Camera>)>,
) {
    let focus = player_transform.translation + Vec3::Y * CAMERA_HEIGHT;
    let forward = camera_transform.forward();

    camera_transform.translation = focus - forward * CAMERA_DISTANCE;
}
