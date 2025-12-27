mod cert;
mod remote;
mod systems;

use crate::Systems;
use crate::network::systems::{receive_updates, setup_connection};
use crate::world::chunk::ChunkEntities;
use bevy::prelude::*;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use voxel_core::VoxelBuffer;
use voxel_net::message::{ClientMessage, ServerMessage};

#[derive(Resource)]
pub struct LocalClientId(pub u32);

#[derive(Default, Resource)]
pub struct PlayerEntities(pub HashMap<u32, Entity>);

#[derive(Resource)]
pub struct TokioRuntime(#[allow(dead_code)] pub Runtime);

#[derive(Resource)]
pub struct Connection {
    outgoing: mpsc::UnboundedSender<ClientMessage>,
    incoming: mpsc::UnboundedReceiver<ServerMessage>,
}

impl Connection {
    pub fn send(&self, msg: ClientMessage) {
        let _ = self.outgoing.send(msg);
    }

    pub fn try_recv(&mut self) -> Option<ServerMessage> {
        self.incoming.try_recv().ok()
    }
}

#[derive(Resource, Default)]
pub struct ChunkLoadQueue(pub VecDeque<(IVec3, Arc<VoxelBuffer>)>);

#[derive(Resource, Default)]
pub struct ChunkUnloadQueue(pub Vec<IVec3>);

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntities>()
            .init_resource::<ChunkLoadQueue>()
            .init_resource::<ChunkUnloadQueue>()
            .init_resource::<ChunkEntities>()
            .add_systems(Startup, setup_connection)
            .add_systems(Update, receive_updates.in_set(Systems::Network));
    }
}
