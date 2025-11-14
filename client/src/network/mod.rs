use bevy::prelude::*;
use shared::Message;
use std::collections::HashMap;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;

pub mod local;
pub mod remote;
mod systems;

#[derive(Default, Resource)]
pub struct PlayerEntities {
    pub map: HashMap<u32, Entity>,
}

#[derive(Resource)]
pub struct LocalClientId(pub u32);

#[allow(dead_code)]
#[derive(Resource)]
pub struct TokioRuntime(pub Runtime);

#[derive(Resource)]
pub struct Server {
    outgoing: mpsc::UnboundedSender<Message>,
    incoming: mpsc::UnboundedReceiver<Message>,
}

impl Server {
    pub fn new(
        outgoing: mpsc::UnboundedSender<Message>,
        incoming: mpsc::UnboundedReceiver<Message>,
    ) -> Self {
        Self { outgoing, incoming }
    }

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
            .add_systems(Startup, systems::setup_server)
            .add_systems(
                Update,
                (
                    local::run_local_simulation.run_if(resource_exists::<local::LocalServer>),
                    systems::receive_updates,
                    systems::send_player_input,
                ),
            );
    }
}
