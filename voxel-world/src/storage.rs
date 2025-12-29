use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use glam::IVec3;
use std::io::{Read, Write};
use std::{env::current_dir, path::PathBuf, sync::Arc};
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use voxel_core::VoxelBuffer;

pub struct Storage {
    dir: PathBuf,
    save_tx: UnboundedSender<(IVec3, Vec<u8>)>,
}

impl Storage {
    pub fn new() -> Self {
        let dir = current_dir().unwrap().join(".store");
        let _ = std::fs::create_dir_all(dir.join("chunks"));
        let _ = std::fs::create_dir_all(dir.join("players"));

        let (save_tx, mut save_rx) = unbounded_channel::<(IVec3, Vec<u8>)>();
        let save_dir = dir.clone();

        std::thread::spawn(move || {
            while let Some((pos, compressed)) = save_rx.blocking_recv() {
                let _ = std::fs::write(
                    save_dir.join(format!("chunks/{}_{}_{}.dat", pos.x, pos.y, pos.z)),
                    compressed,
                );
            }
        });

        Self { dir, save_tx }
    }

    pub fn get_chunk(&self, pos: IVec3) -> Option<Arc<VoxelBuffer>> {
        let compressed = std::fs::read(
            self.dir
                .join(format!("chunks/{}_{}_{}.dat", pos.x, pos.y, pos.z)),
        )
        .ok()?;

        let mut decoder = ZlibDecoder::new(&compressed[..]);
        let mut data = Vec::new();
        decoder.read_to_end(&mut data).ok()?;

        postcard::from_bytes(&data).ok().map(Arc::new)
    }

    pub fn save_chunk(&self, pos: IVec3, buffer: &VoxelBuffer) {
        if let Ok(data) = postcard::to_allocvec(buffer) {
            let mut compressed = Vec::new();
            let mut encoder = ZlibEncoder::new(&mut compressed, Compression::fast());

            if encoder.write_all(&data).is_ok() && encoder.finish().is_ok() {
                let _ = self.save_tx.send((pos, compressed));
            }
        }
    }
}
