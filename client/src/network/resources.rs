use bevy::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;

#[derive(Default, Resource)]
pub struct PlayerEntities {
    pub map: HashMap<u32, Entity>,
}

#[derive(Resource)]
pub struct LocalClientId(pub u32);

#[allow(dead_code)]
#[derive(Resource)]
pub struct TokioRuntime(pub Runtime);
