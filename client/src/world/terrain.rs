use std::sync::Arc;

use bevy::prelude::*;
use noise::{core::perlin::perlin_2d, permutationtable::PermutationTable};
use voxel_core::Voxel;

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

pub fn generate_terrain(coord: IVec3, noise_table: Arc<PermutationTable>) -> ChunkVoxels {
    let mut voxels = ChunkVoxels::new();

    let scale = 0.02;
    let height_scale = 30.0;
    let base_height = 32.0;

    for x in 0..CHUNK_SIZE {
        for z in 0..CHUNK_SIZE {
            let world_x = coord.x * CHUNK_SIZE as i32 + x as i32;
            let world_z = coord.z * CHUNK_SIZE as i32 + z as i32;

            let noise_value = perlin_2d(
                [world_x as f64 * scale, world_z as f64 * scale].into(),
                noise_table.as_ref(),
            );
            let height = base_height + (noise_value * height_scale);
            let height_i = height as i32;

            for y in 0..CHUNK_SIZE {
                let world_y = coord.y * CHUNK_SIZE as i32 + y as i32;

                let voxel_type = if world_y < height_i - 4 {
                    Voxel::STONE
                } else if world_y < height_i {
                    Voxel::DIRT
                } else {
                    Voxel::EMPTY
                };

                let pos = UVec3::new(x as u32, y as u32, z as u32);

                voxels.data.set(pos, voxel_type);
            }
        }
    }

    voxels
}
