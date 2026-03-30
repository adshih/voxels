use std::sync::{Arc, OnceLock};

use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use voxel_world::{
    VoxelWorld, bridge::Bridge, command::WorldCommand, envelope::Envelope, event::WorldEvent,
};

pub fn host(name: String) -> anyhow::Result<(u32, Bridge)> {
    let world = VoxelWorld::new(123);

    let (cmd_tx, cmd_rx) = unbounded_channel();
    let (req_tx, req_rx) = unbounded_channel();
    let (event_tx, event_rx) = unbounded_channel();

    // yuck!
    let from = Arc::new(OnceLock::new());
    let cmd_rx = pack(from.clone(), cmd_rx);
    let event_rx = unpack(event_rx);

    std::thread::spawn(move || world.run(cmd_rx, req_rx, event_tx));

    let bridge = Bridge::new(cmd_tx, req_tx, event_rx);
    let id = bridge.connect(name)?;
    from.set(id).unwrap();

    Ok((id, bridge))
}

fn pack(
    id: Arc<OnceLock<u32>>,
    mut cmd_rx: UnboundedReceiver<WorldCommand>,
) -> UnboundedReceiver<Envelope<WorldCommand>> {
    let (packed_tx, packed_rx) = unbounded_channel();

    std::thread::spawn(move || {
        let id = *id.wait();
        while let Some(cmd) = cmd_rx.blocking_recv() {
            if packed_tx.send(Envelope::from(id, cmd)).is_err() {
                break;
            }
        }
    });

    packed_rx
}

fn unpack(mut event_rx: UnboundedReceiver<Envelope<WorldEvent>>) -> UnboundedReceiver<WorldEvent> {
    let (unpacked_tx, unpacked_rx) = unbounded_channel();

    std::thread::spawn(move || {
        while let Some(event) = event_rx.blocking_recv() {
            if unpacked_tx.send(event.payload).is_err() {
                break;
            }
        }
    });

    unpacked_rx
}
