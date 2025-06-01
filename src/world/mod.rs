mod core;

use bevy::prelude::*;

#[derive(Resource)]
struct WorldSettings {}

impl Default for WorldSettings {
    fn default() -> Self {
        Self {}
    }
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, _app: &mut App) {}
}
