use std::collections::HashMap;

use bevy::prelude::*;

#[derive(Default, Resource)]
pub struct PlayerEntities {
    _map: HashMap<u32, Entity>,
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntities>();
    }
}
