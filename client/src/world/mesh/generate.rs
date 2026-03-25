use std::sync::Arc;

use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use voxel_core::{
    VoxelBuffer,
    mesh::{MeshBuffer, Mesher, block::BlockMesher},
};

pub fn generate_mesh(buf: Arc<VoxelBuffer>) -> Option<Mesh> {
    BlockMesher.generate(&buf).map(into_bevy_mesh)
}

fn into_bevy_mesh(buf: MeshBuffer) -> Mesh {
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, buf.positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, buf.normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, buf.uvs)
    .with_inserted_indices(Indices::U32(buf.indices))
}
