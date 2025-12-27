use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};
use std::sync::Arc;
use voxel_core::{Voxel, VoxelBuffer};

const ATLAS_SIZE: f32 = 16.0;
const TEXTURE_SIZE: f32 = 1.0 / ATLAS_SIZE;
const BASE_UVS: [[f32; 2]; 4] = [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]];
const QUAD_INDICES: [u32; 6] = [0, 1, 2, 0, 2, 3];

#[derive(Clone, Copy)]
enum CubeFace {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

const CUBE_FACES: [CubeFace; 6] = [
    CubeFace::Front,
    CubeFace::Back,
    CubeFace::Right,
    CubeFace::Left,
    CubeFace::Top,
    CubeFace::Bottom,
];

impl CubeFace {
    fn offset(self) -> IVec3 {
        match self {
            CubeFace::Front => IVec3::new(0, 0, 1),
            CubeFace::Back => IVec3::new(0, 0, -1),
            CubeFace::Right => IVec3::new(1, 0, 0),
            CubeFace::Left => IVec3::new(-1, 0, 0),
            CubeFace::Top => IVec3::new(0, 1, 0),
            CubeFace::Bottom => IVec3::new(0, -1, 0),
        }
    }

    fn normal(self) -> [f32; 3] {
        self.offset().as_vec3().to_array()
    }

    fn vertices(self, pos: Vec3) -> [[f32; 3]; 4] {
        let Vec3 { x, y, z } = pos;
        match self {
            CubeFace::Front => [
                [x, y, z + 1.0],
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x, y + 1.0, z + 1.0],
            ],
            CubeFace::Back => [
                [x + 1.0, y, z],
                [x, y, z],
                [x, y + 1.0, z],
                [x + 1.0, y + 1.0, z],
            ],
            CubeFace::Right => [
                [x + 1.0, y, z + 1.0],
                [x + 1.0, y, z],
                [x + 1.0, y + 1.0, z],
                [x + 1.0, y + 1.0, z + 1.0],
            ],
            CubeFace::Left => [
                [x, y, z],
                [x, y, z + 1.0],
                [x, y + 1.0, z + 1.0],
                [x, y + 1.0, z],
            ],
            CubeFace::Top => [
                [x, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z + 1.0],
                [x + 1.0, y + 1.0, z],
                [x, y + 1.0, z],
            ],
            CubeFace::Bottom => [
                [x, y, z],
                [x + 1.0, y, z],
                [x + 1.0, y, z + 1.0],
                [x, y, z + 1.0],
            ],
        }
    }
}

pub fn generate_mesh(buffer: Arc<VoxelBuffer>) -> Option<Mesh> {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    let size = buffer.size;

    for x in 0..size.x {
        for y in 0..size.y {
            for z in 0..size.z {
                let pos = UVec3::new(x, y, z);
                let voxel = buffer.get(pos);

                if !voxel.is_empty() {
                    for face in CUBE_FACES {
                        if should_render_face(&buffer, pos, face) {
                            add_face(
                                &mut positions,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                pos.as_vec3(),
                                face,
                                voxel,
                            );
                        }
                    }
                }
            }
        }
    }

    if positions.is_empty() {
        return None;
    }

    Some(
        Mesh::new(
            PrimitiveTopology::TriangleList,
            RenderAssetUsages::default(),
        )
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
        .with_inserted_indices(Indices::U32(indices)),
    )
}

fn should_render_face(voxels: &VoxelBuffer, pos: UVec3, face: CubeFace) -> bool {
    let neighbor = pos.as_ivec3() + face.offset();

    if neighbor.x < 0
        || neighbor.x >= voxels.size.x as i32
        || neighbor.y < 0
        || neighbor.y >= voxels.size.y as i32
        || neighbor.z < 0
        || neighbor.z >= voxels.size.z as i32
    {
        return true;
    }

    voxels.get(neighbor.as_uvec3()).is_empty()
}

fn add_face(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    pos: Vec3,
    face: CubeFace,
    voxel: Voxel,
) {
    let base_index = positions.len() as u32;
    let normal = face.normal();
    let texture_coords = get_texture_coords(voxel);

    positions.extend_from_slice(&face.vertices(pos));
    normals.extend([normal; 4]);

    for &base_uv in &BASE_UVS {
        uvs.push([
            texture_coords.0 + base_uv[0] * TEXTURE_SIZE,
            texture_coords.1 + base_uv[1] * TEXTURE_SIZE,
        ]);
    }

    for &offset in &QUAD_INDICES {
        indices.push(base_index + offset);
    }
}

fn get_texture_coords(voxel: Voxel) -> (f32, f32) {
    match voxel {
        Voxel::STONE => (0.0 * TEXTURE_SIZE, 0.0 * TEXTURE_SIZE),
        Voxel::DIRT => (1.0 * TEXTURE_SIZE, 0.0 * TEXTURE_SIZE),
        Voxel::EMPTY => (0.0, 0.0),
        _ => (15.0 * TEXTURE_SIZE, 15.0 * TEXTURE_SIZE),
    }
}
