pub mod block;

use crate::VoxelBuffer;

pub struct MeshBuffer {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

pub trait Mesher: Send + Sync {
    fn generate(&self, buffer: &VoxelBuffer) -> Option<MeshBuffer>;
}
