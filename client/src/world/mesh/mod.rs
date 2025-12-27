mod generate;

use crate::world::{ChunkData, MAX_MESH_TASKS, mesh::generate::generate_mesh};
use bevy::{
    prelude::*,
    tasks::{AsyncComputeTaskPool, Task, block_on, poll_once},
};

#[derive(Component)]
pub struct NeedsMesh;

#[derive(Component)]
pub struct MeshTask(Task<Option<Mesh>>);

#[derive(Component)]
pub struct MeshReady(Option<Mesh>);

#[derive(Resource)]
pub struct BlockMaterial(Handle<StandardMaterial>);

pub fn load_assets(
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut commands: Commands,
) {
    let texture = asset_server.load("blocks.png");
    let material = materials.add(StandardMaterial {
        base_color_texture: Some(texture.clone()),
        unlit: false,
        ..default()
    });

    commands.insert_resource(BlockMaterial(material));
}

pub fn queue_mesh_tasks(
    mut commands: Commands,
    chunk_data: Query<(Entity, &ChunkData), With<NeedsMesh>>,
    active_mesh_tasks: Query<(), With<MeshTask>>,
) {
    let active_count = active_mesh_tasks.iter().count();
    if active_count >= MAX_MESH_TASKS {
        return;
    }

    let pool = AsyncComputeTaskPool::get();

    for (entity, ChunkData(buffer)) in chunk_data.iter().take(MAX_MESH_TASKS - active_count) {
        let buffer = buffer.clone();
        let task = pool.spawn(async move { generate_mesh(buffer) });
        commands
            .entity(entity)
            .remove::<NeedsMesh>()
            .insert(MeshTask(task));
    }
}

pub fn collect_mesh_tasks(mut commands: Commands, mut tasks: Query<(Entity, &mut MeshTask)>) {
    for (entity, mut task) in tasks.iter_mut() {
        if let Some(result) = block_on(poll_once(&mut task.0)) {
            commands.entity(entity).remove::<MeshTask>();
            if let Some(mesh) = result {
                commands.entity(entity).insert(MeshReady(Some(mesh)));
            }
        }
    }
}

pub fn upload_meshes(
    mut commands: Commands,
    mut ready: Query<(Entity, &mut MeshReady)>,
    mut meshes: ResMut<Assets<Mesh>>,
    block_material: Res<BlockMaterial>,
) {
    for (entity, mut mesh_ready) in ready.iter_mut().take(MAX_MESH_TASKS) {
        if let Some(mesh) = mesh_ready.0.take() {
            commands.entity(entity).remove::<MeshReady>().insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(block_material.0.clone()),
            ));
        }
    }
}
