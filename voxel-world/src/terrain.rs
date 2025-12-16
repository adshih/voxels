use std::{collections::HashMap, sync::Arc};

use glam::{IVec3, UVec3, Vec3};
use noise::{core::perlin::perlin_2d, permutationtable::PermutationTable};
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

pub struct VoxelTerrain {
    generator: TerrainGenerator,
    chunks: HashMap<IVec3, Arc<VoxelBuffer>>,
}

impl VoxelTerrain {
    pub fn new(seed: u32) -> Self {
        Self {
            generator: TerrainGenerator::new(seed),
            chunks: HashMap::new(),
        }
    }

    pub fn load_chunk(&mut self, pos: IVec3) -> Arc<VoxelBuffer> {
        self.chunks
            .entry(pos)
            .or_insert_with(|| Arc::new(self.generator.generate(pos)))
            .clone()
    }

    pub fn load_chunks_around(&mut self, center: IVec3) -> Vec<(IVec3, Arc<VoxelBuffer>)> {
        let radius = CHUNK_RENDER_DISTANCE;
        let mut loaded = Vec::new();

        for x in -radius..=radius {
            for y in -radius..=radius {
                for z in -radius..=radius {
                    let pos = IVec3::new(center.x + x, center.y + y, center.z + z);
                    let data = self.load_chunk(pos);
                    loaded.push((pos, data));
                }
            }
        }

        loaded
    }
}

pub fn world_to_chunk_pos(pos: Vec3) -> IVec3 {
    IVec3::new(
        (pos.x / CHUNK_SIZE.x as f32).floor() as i32,
        (pos.y / CHUNK_SIZE.y as f32).floor() as i32,
        (pos.z / CHUNK_SIZE.z as f32).floor() as i32,
    )
}
