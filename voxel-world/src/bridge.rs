use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::{
    command::WorldCommand,
    event::WorldEvent,
    request::{Call, Connect, PendingRequest, Ping, Pong},
};

pub struct Bridge {
    cmd_tx: UnboundedSender<WorldCommand>,
    req_tx: UnboundedSender<PendingRequest>,
    event_rx: UnboundedReceiver<WorldEvent>,
}

impl Bridge {
    pub fn new(
        cmd_tx: UnboundedSender<WorldCommand>,
        req_tx: UnboundedSender<PendingRequest>,
        event_rx: UnboundedReceiver<WorldEvent>,
    ) -> Self {
        Self {
            cmd_tx,
            req_tx,
            event_rx,
        }
    }

    pub fn send(&self, cmd: impl Into<WorldCommand>) {
        let _ = self.cmd_tx.send(cmd.into());
    }

    pub fn try_recv(&mut self) -> Option<WorldEvent> {
        self.event_rx.try_recv().ok()
    }

    pub fn connect(&self, name: String) -> anyhow::Result<u32> {
        let (call, rx) = Call::new(Connect { name });
        let _ = self.req_tx.send(PendingRequest::Connect(call));
        Ok(rx.blocking_recv()?)
    }

    pub fn _ping(&self) -> anyhow::Result<Pong> {
        let (call, rx) = Call::new(Ping);
        let _ = self.req_tx.send(PendingRequest::Ping(call));
        Ok(rx.blocking_recv()?)
    }
}
