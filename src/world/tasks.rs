use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use super::events::*;
use crate::world::*;

#[derive(Component)]
pub struct ChunkGenerationTask(Task<ChunkVoxels>);

#[derive(Component)]
pub struct ChunkMeshTask {
    task: Task<Mesh>,
    voxel_version: u32,
}

pub fn start_generation_tasks(
    mut commands: Commands,
    mut events: EventReader<ChunkNeedsGeneration>,
) {
    let pool = AsyncComputeTaskPool::get();

    for event in events.read() {
        let coord = event.coord;
        let task = pool.spawn(async move { generate_terrain(coord) });

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
            priority: MeshPriority::default(),
        });
    }
}

pub fn start_mesh_tasks(
    mut commands: Commands,
    mut events: EventReader<ChunkNeedsMesh>,
    chunks: Query<&ChunkVoxels, Without<ChunkMeshTask>>,
) {
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
            let handle = meshes.add(mesh);
            let version = task.voxel_version;

            commands.entity(entity).remove::<ChunkMeshTask>();

            ready_events.write(ChunkMeshReady {
                entity,
                mesh: handle,
                voxel_version: version,
            });
        }
    }
}

pub fn apply_mesh_to_world(
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
                    priority: MeshPriority::default(),
                });
            }
        }
    }
}

fn generate_terrain(coord: ChunkCoord) -> ChunkVoxels {
    let mut voxels = ChunkVoxels::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let world_y = coord.0.y * CHUNK_SIZE as i32 + y as i32;

                let voxel_type = if world_y < 16 {
                    VoxelType::STONE
                } else if world_y < 20 {
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

fn generate_mesh(voxels: ChunkVoxels) -> Mesh {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                if voxels.get(x, y, z).is_solid() {
                    add_cube_to_mesh(
                        &mut positions,
                        &mut indices,
                        &mut normals,
                        [x as f32, y as f32, z as f32],
                    );
                }
            }
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn add_cube_to_mesh(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    pos: [f32; 3],
) {
    let base_index = positions.len() as u32;

    let verts = [
        [pos[0], pos[1], pos[2]],
        [pos[0] + 1.0, pos[1], pos[2]],
        [pos[0] + 1.0, pos[1] + 1.0, pos[2]],
        [pos[0], pos[1] + 1.0, pos[2]],
    ];

    positions.extend_from_slice(&verts);

    for _ in 0..4 {
        normals.push([0.0, 0.0, 1.0]);
    }

    indices.extend_from_slice(&[
        base_index,
        base_index + 1,
        base_index + 2,
        base_index,
        base_index + 2,
        base_index + 3,
    ]);
}
