pub mod bridge;
mod cert;
mod local;
mod quic;

use bevy::prelude::*;
use voxel_world::event::*;

use crate::{Settings, connection::bridge::{FromWorld, WorldBridge}, player};

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_connection)
            .add_systems(Update, dispatch_world_events);
    }
}

fn setup_connection(mut commands: Commands, settings: Res<Settings>) {
    let (id, bridge) = match &settings.addr {
        Some(addr) => quic::connect(addr.clone(), settings.name.clone()),
        None => local::host(settings.name.clone()),
    }
    .expect("Failed to start world connection");

    let name = settings.name.clone();
    commands.trigger(player::Connected { id, name });
    commands.insert_resource(WorldBridge(bridge));
}

fn dispatch_world_events(mut commands: Commands, mut world: ResMut<WorldBridge>) {
    while let Some(msg) = world.try_recv() {
        match msg {
            WorldEvent::PlayerJoined(e) => commands.trigger(FromWorld(e)),
            WorldEvent::PlayerLeft(e) => commands.trigger(FromWorld(e)),
            WorldEvent::PlayerMoved(e) => commands.trigger(FromWorld(e)),
            WorldEvent::ChunkLoaded(e) => commands.trigger(FromWorld(e)),
            WorldEvent::ChunkUnloaded(e) => commands.trigger(FromWorld(e)),
        }
    }
}
