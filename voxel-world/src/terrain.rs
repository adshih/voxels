use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use glam::{IVec3, UVec3, Vec3};
use noise::{core::perlin::perlin_2d, permutationtable::PermutationTable};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use voxel_core::{Voxel, VoxelBuffer};

pub const CHUNK_SIZE: UVec3 = UVec3::splat(32);
pub const CHUNK_RENDER_DISTANCE: i32 = 12;

struct TerrainGenerator {
    seed_table: PermutationTable,
}

impl TerrainGenerator {
    fn new(seed: u32) -> Self {
        Self {
            seed_table: PermutationTable::new(seed),
        }
    }

    pub fn generate(&self, pos: IVec3) -> VoxelBuffer {
        let mut buffer = VoxelBuffer::new(CHUNK_SIZE);

        let scale = 0.02;
        let height_scale = 30.0;
        let base_height = 32.0;

        for x in 0..CHUNK_SIZE.x {
            for z in 0..CHUNK_SIZE.z {
                let world_x = pos.x * CHUNK_SIZE.x as i32 + x as i32;
                let world_z = pos.z * CHUNK_SIZE.z as i32 + z as i32;

                let noise_value = perlin_2d(
                    [world_x as f64 * scale, world_z as f64 * scale].into(),
                    &self.seed_table,
                );
                let height = base_height + (noise_value * height_scale);
                let height_i = height as i32;

                for y in 0..CHUNK_SIZE.y {
                    let world_y = pos.y * CHUNK_SIZE.y as i32 + y as i32;

                    let voxel = if world_y < height_i - 4 {
                        Voxel::STONE
                    } else if world_y < height_i {
                        Voxel::DIRT
                    } else {
                        Voxel::EMPTY
                    };

                    let pos = UVec3::new(x as u32, y as u32, z as u32);

                    buffer.set(pos, voxel);
                }
            }
        }

        buffer
    }
}

pub struct Terrain {
    chunks: HashMap<IVec3, Arc<VoxelBuffer>>,
    pending: HashSet<IVec3>,
    request_tx: UnboundedSender<IVec3>,
    result_rx: UnboundedReceiver<(IVec3, Arc<VoxelBuffer>)>,
}

impl Terrain {
    pub fn new(seed: u32) -> Self {
        let (request_tx, mut request_rx) = unbounded_channel();
        let (result_tx, result_rx) = unbounded_channel();

        std::thread::spawn(move || {
            let generator = TerrainGenerator::new(seed);

            while let Some(pos) = request_rx.blocking_recv() {
                let data = Arc::new(generator.generate(pos));
                let _ = result_tx.send((pos, data));
            }
        });

        Terrain {
            chunks: HashMap::new(),
            pending: HashSet::new(),
            request_tx,
            result_rx,
        }
    }

    pub fn _get(&self, pos: IVec3) -> Option<Arc<VoxelBuffer>> {
        self.chunks.get(&pos).cloned()
    }

    pub fn request(&mut self, pos: IVec3) {
        if !self.chunks.contains_key(&pos) && !self.pending.contains(&pos) {
            self.pending.insert(pos);
            let _ = self.request_tx.send(pos);
        }
    }

    pub fn poll(&mut self) -> Vec<(IVec3, Arc<VoxelBuffer>)> {
        let mut ready = Vec::new();

        while let Ok((pos, data)) = self.result_rx.try_recv() {
            self.chunks.insert(pos, data.clone());
            self.pending.remove(&pos);
            ready.push((pos, data));
        }

        ready
    }
}

pub fn world_to_chunk_pos(pos: Vec3) -> IVec3 {
    IVec3::new(
        (pos.x / CHUNK_SIZE.x as f32).floor() as i32,
        (pos.y / CHUNK_SIZE.y as f32).floor() as i32,
        (pos.z / CHUNK_SIZE.z as f32).floor() as i32,
    )
}

pub fn chunks_in_radius(center: IVec3, radius: i32) -> Vec<IVec3> {
    let mut positions = Vec::new();

    for x in -radius..=radius {
        for y in -radius..=radius {
            for z in -radius..=radius {
                positions.push(center + IVec3::new(x, y, z));
            }
        }
    }
    positions
}
