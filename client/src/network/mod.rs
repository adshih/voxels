use bevy::prelude::*;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

use server::Message;

use crate::network::systems::ChunkLoadQueue;

mod cert;
pub mod remote;
pub mod systems;

#[derive(Default, Resource)]
pub struct PlayerEntities {
    pub map: HashMap<u32, Entity>,
}

#[derive(Resource)]
pub struct LocalClientId(pub u32);

#[derive(Resource)]
pub struct TokioRuntime(#[allow(dead_code)] pub Runtime);

#[derive(Resource)]
pub struct Connection {
    outgoing: mpsc::UnboundedSender<Message>,
    incoming: mpsc::UnboundedReceiver<Message>,
}

impl Connection {
    pub fn send(&self, msg: Message) {
        let _ = self.outgoing.send(msg);
    }

    pub fn try_recv(&mut self) -> Option<Message> {
        self.incoming.try_recv().ok()
    }
}

pub struct NetworkPlugin;

impl Plugin for NetworkPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerEntities>()
            .init_resource::<ChunkLoadQueue>()
            .add_systems(Startup, systems::setup_connection)
            .add_systems(
                Update,
                (
                    systems::receive_updates,
                    systems::send_player_input,
                    systems::process_chunk_load_queue,
                ),
            );
    }
}
