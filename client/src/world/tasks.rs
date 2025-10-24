use std::sync::Arc;

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use super::events::*;
use crate::world::*;

const MAX_GENERATION_TASKS: usize = 3;
const MAX_MESH_TASKS: usize = 1;

#[derive(Component)]
pub struct ChunkGenerationTask(Task<ChunkVoxels>);

#[derive(Component)]
pub struct ChunkMeshTask {
    task: Task<Option<Mesh>>,
    voxel_version: u32,
}

pub fn start_generation_tasks(
    mut commands: Commands,
    mut events: EventReader<ChunkNeedsGeneration>,
    active_tasks: Query<&ChunkGenerationTask>,
    terrain_noise: Res<TerrainNoise>,
) {
    if active_tasks.iter().count() >= MAX_GENERATION_TASKS {
        return;
    }

    let pool = AsyncComputeTaskPool::get();

    for event in events.read() {
        let coord = event.coord;
        let noise_table = Arc::clone(&terrain_noise.table);

        let task = pool.spawn(async move { generate_terrain(coord, noise_table) });

        commands
            .entity(event.entity)
            .insert(ChunkGenerationTask(task));
    }
}

pub fn complete_generation_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkGenerationTask, &Chunk)>,
    mut ready_events: EventWriter<ChunkVoxelsReady>,
) {
    for (entity, mut task, chunk) in tasks.iter_mut() {
        if let Some(voxels) = future::block_on(future::poll_once(&mut task.0)) {
            commands.entity(entity).remove::<ChunkGenerationTask>();
            commands.entity(entity).insert(voxels);

            ready_events.write(ChunkVoxelsReady {
                entity,
                coord: chunk.coord,
            });
        }
    }
}

pub fn route_voxels_to_mesh(
    mut voxel_events: EventReader<ChunkVoxelsReady>,
    mut mesh_events: EventWriter<ChunkNeedsMesh>,
) {
    for event in voxel_events.read() {
        mesh_events.write(ChunkNeedsMesh {
            entity: event.entity,
            coord: event.coord,
        });
    }
}

pub fn start_mesh_tasks(
    mut commands: Commands,
    mut events: EventReader<ChunkNeedsMesh>,
    chunks: Query<&ChunkVoxels, Without<ChunkMeshTask>>,
    active_tasks: Query<&ChunkMeshTask>,
) {
    if active_tasks.iter().count() >= MAX_MESH_TASKS {
        return;
    }

    let pool = AsyncComputeTaskPool::get();

    for event in events.read() {
        if let Ok(voxels) = chunks.get(event.entity) {
            let voxels_copy = voxels.clone();
            let version = voxels.version;

            let task = pool.spawn(async move { generate_mesh(voxels_copy) });

            commands.entity(event.entity).insert(ChunkMeshTask {
                task,
                voxel_version: version,
            });
        }
    }
}

pub fn complete_mesh_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut ChunkMeshTask)>,
    mut ready_events: EventWriter<ChunkMeshReady>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(mesh) = future::block_on(future::poll_once(&mut task.task)) {
            commands.entity(entity).remove::<ChunkMeshTask>();

            if let Some(mesh) = mesh {
                let handle = meshes.add(mesh);
                ready_events.write(ChunkMeshReady {
                    entity,
                    mesh: handle,
                    voxel_version: task.voxel_version,
                });
            }
        }
    }
}

pub fn validate_mesh_versions(
    mut commands: Commands,
    mut events: EventReader<ChunkMeshReady>,
    chunks: Query<(&Chunk, &ChunkVoxels)>,
    mut mesh_events: EventWriter<ChunkNeedsMesh>,
) {
    for event in events.read() {
        if let Ok((chunk, voxels)) = chunks.get(event.entity) {
            if event.voxel_version == voxels.version {
                commands.entity(event.entity).insert(ChunkMesh {
                    handle: event.mesh.clone(),
                    voxel_version: event.voxel_version,
                });
            } else {
                mesh_events.write(ChunkNeedsMesh {
                    entity: event.entity,
                    coord: chunk.coord,
                });
            }
        }
    }
}

pub fn render_chunks(
    mut commands: Commands,
    changed_meshes: Query<(Entity, &ChunkMesh), Changed<ChunkMesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    atlas_handle: Res<AtlasHandle>,
) {
    for (entity, chunk_mesh) in changed_meshes.iter() {
        let material = materials.add(StandardMaterial {
            base_color_texture: Some(atlas_handle.0.clone()),
            unlit: false,
            ..default()
        });

        commands
            .entity(entity)
            .insert((Mesh3d(chunk_mesh.handle.clone()), MeshMaterial3d(material)));
    }
}
