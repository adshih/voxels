use std::sync::Arc;

use bevy::prelude::*;
use noise::{core::perlin::perlin_2d, permutationtable::PermutationTable};

use crate::world::*;

#[derive(Resource)]
pub struct TerrainNoise {
    pub table: Arc<PermutationTable>,
}

impl Default for TerrainNoise {
    fn default() -> Self {
        Self {
            table: Arc::new(PermutationTable::new(123)),
        }
    }
}

pub fn generate_terrain(coord: ChunkCoord, noise_table: Arc<PermutationTable>) -> ChunkVoxels {
    let mut voxels = ChunkVoxels::new();

    let scale = 0.02;
    let height_scale = 30.0;
    let base_height = 32.0;

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let world_x = coord.0.x * CHUNK_SIZE as i32 + x as i32;
            let world_z = coord.0.z * CHUNK_SIZE as i32 + z as i32;

            let noise_value = perlin_2d(
                [world_x as f64 * scale, world_z as f64 * scale].into(),
                noise_table.as_ref(),
            );
            let height = base_height + (noise_value * height_scale);
            let height_i = height as i32;

            for y in 0..CHUNK_SIZE {
                let world_y = coord.0.y * CHUNK_SIZE as i32 + y as i32;

                let voxel_type = if world_y < height_i - 4 {
                    VoxelType::STONE
                } else if world_y < height_i {
                    VoxelType::DIRT
                } else {
                    VoxelType::AIR
                };

                voxels.set(x, y, z, voxel_type);
            }
        }
    }

    voxels
}
