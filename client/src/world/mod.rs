pub mod chunk;
pub mod mesh;

use crate::{
    Systems,
    world::{chunk::*, mesh::*},
};
use bevy::prelude::*;

pub const MAX_CHUNK_LOAD_PER_FRAME: usize = 20;
pub const MAX_MESH_TASKS: usize = 100;

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ChunkEntities>()
            .add_systems(Startup, load_assets)
            .add_systems(
                Update,
                (process_chunk_load_queue, process_chunk_unload_queue),
            )
            .add_systems(
                Update,
                (queue_mesh_tasks, collect_mesh_tasks, upload_meshes)
                    .chain()
                    .in_set(Systems::Mesh),
            );
    }
}
