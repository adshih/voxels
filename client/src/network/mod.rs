use bevy::prelude::*;

use crate::network::resources::PlayerEntities;

pub mod client;
mod resources;
mod systems;

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntities>()
            .add_systems(Startup, systems::setup_network)
            .add_systems(
                Update,
                (systems::handle_network_events, systems::send_input),
            );
    }
}
