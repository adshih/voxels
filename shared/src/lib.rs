use bevy::prelude::*;

pub const MOVEMENT_SPEED: f32 = 10.0;
pub const SPRINT_MULTIPLIER: f32 = 2.0;

#[derive(Default, Debug, Copy, Clone, Component)]
pub struct PlayerInput {
    pub forward: f32,
    pub right: f32,
    pub up: f32,
    pub sprint: bool,
    pub camera_forward: Vec3,
}

pub fn calculate_movement(
    input: &PlayerInput,
    current_position: Vec3,
    camera_forward: Vec3,
    delta_time: f32,
) -> Vec3 {
    if input.forward == 0.0 && input.right == 0.0 && input.up == 0.0 {
        return current_position;
    }

    let forward = Vec3::new(camera_forward.x, 0.0, camera_forward.z).normalize_or_zero();
    let right = forward.cross(Vec3::Y).normalize_or_zero();

    let mut velocity = forward * input.forward + right * input.right + Vec3::Y * input.up;
    velocity = velocity.normalize_or_zero();

    let speed = if input.sprint {
        MOVEMENT_SPEED * SPRINT_MULTIPLIER
    } else {
        MOVEMENT_SPEED
    };

    current_position + velocity * speed * delta_time
}
