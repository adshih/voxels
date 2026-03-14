use bevy::prelude::*;
use voxel_world::Bridge;

#[derive(Event, Deref)]
pub struct FromWorld<T: Send + Sync + 'static>(pub T);

#[derive(Resource, Deref, DerefMut)]
pub struct WorldBridge(pub Bridge);
