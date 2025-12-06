mod chunk;
mod events;
mod mesh;
mod tasks;
mod terrain;

pub use chunk::*;
pub use events::*;
pub use mesh::*;
use tasks::*;
pub use terrain::*;

use bevy::prelude::*;

#[derive(Resource)]
pub struct WorldSettings {
    render_distance: u8,
}

impl Default for WorldSettings {
    fn default() -> Self {
        Self { render_distance: 6 }
    }
}

#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ChunkPipeline {
    Management,
    Generation,
    Meshing,
    Completion,
}

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WorldSettings>()
            .init_resource::<ChunkManager>()
            .init_resource::<TerrainNoise>()
            .add_message::<ChunkNeedsGeneration>()
            .add_message::<ChunkVoxelsReady>()
            .add_message::<ChunkNeedsMesh>()
            .add_message::<ChunkMeshReady>()
            .configure_sets(
                Update,
                (
                    ChunkPipeline::Management,
                    ChunkPipeline::Generation,
                    ChunkPipeline::Meshing,
                    ChunkPipeline::Completion,
                )
                    .chain(),
            )
            .add_systems(Startup, load_assets)
            .add_systems(
                Update,
                (
                    (queue_chunk_operations, process_chunk_operations)
                        .chain()
                        .in_set(ChunkPipeline::Management),
                    (
                        start_generation_tasks,
                        complete_generation_tasks,
                        route_voxels_to_mesh,
                    )
                        .chain()
                        .in_set(ChunkPipeline::Generation),
                    (start_mesh_tasks, complete_mesh_tasks)
                        .chain()
                        .in_set(ChunkPipeline::Meshing),
                    (validate_mesh_versions, render_chunks)
                        .chain()
                        .in_set(ChunkPipeline::Completion),
                ),
            );
    }
}

#[derive(Resource)]
pub struct AtlasHandle(Handle<Image>);

fn load_assets(asset_server: Res<AssetServer>, mut commands: Commands) {
    let handle = asset_server.load("blocks.png");
    commands.insert_resource(AtlasHandle(handle));
}
