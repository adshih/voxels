use bevy::prelude::*;

#[derive(Message)]
pub struct ChunkNeedsGeneration {
    pub entity: Entity,
    pub coord: IVec3,
}

#[derive(Message)]
pub struct ChunkVoxelsReady {
    pub entity: Entity,
}

#[derive(Message)]
pub struct ChunkNeedsMesh {
    pub entity: Entity,
}

#[derive(Message)]
pub struct ChunkMeshReady {
    pub entity: Entity,
    pub mesh: Handle<Mesh>,
    pub voxel_version: u32,
}
