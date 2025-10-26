use super::*;
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
};

const CUBE_FACES: [CubeFace; 6] = [
    CubeFace::Front,
    CubeFace::Back,
    CubeFace::Right,
    CubeFace::Left,
    CubeFace::Top,
    CubeFace::Bottom,
];

const ATLAS_SIZE: f32 = 16.0;
const TEXTURE_SIZE: f32 = 1.0 / ATLAS_SIZE;

#[derive(Clone, Copy)]
enum CubeFace {
    Front,
    Back,
    Right,
    Left,
    Top,
    Bottom,
}

impl CubeFace {
    fn normal(self) -> [f32; 3] {
        match self {
            CubeFace::Front => [0.0, 0.0, 1.0],
            CubeFace::Back => [0.0, 0.0, -1.0],
            CubeFace::Right => [1.0, 0.0, 0.0],
            CubeFace::Left => [-1.0, 0.0, 0.0],
            CubeFace::Top => [0.0, 1.0, 0.0],
            CubeFace::Bottom => [0.0, -1.0, 0.0],
        }
    }

    fn vertices(self, pos: [f32; 3]) -> [[f32; 3]; 4] {
        let [x, y, z] = pos;
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

    fn neighbor_offset(self) -> (i32, i32, i32) {
        match self {
            CubeFace::Front => (0, 0, 1),
            CubeFace::Back => (0, 0, -1),
            CubeFace::Right => (1, 0, 0),
            CubeFace::Left => (-1, 0, 0),
            CubeFace::Top => (0, 1, 0),
            CubeFace::Bottom => (0, -1, 0),
        }
    }

    fn base_uvs(self) -> [[f32; 2]; 4] {
        [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]
    }
}

pub fn generate_mesh(voxels: ChunkVoxels) -> Option<Mesh> {
    let mut positions = Vec::new();
    let mut indices = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();

    for x in 0..CHUNK_SIZE {
        for y in 0..CHUNK_SIZE {
            for z in 0..CHUNK_SIZE {
                let voxel = voxels.get(x, y, z);
                if voxel.is_solid() {
                    for face in CUBE_FACES {
                        if should_render_face(&voxels, x, y, z, face) {
                            add_face_to_mesh(
                                &mut positions,
                                &mut indices,
                                &mut normals,
                                &mut uvs,
                                [x as f32, y as f32, z as f32],
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

fn should_render_face(voxels: &ChunkVoxels, x: usize, y: usize, z: usize, face: CubeFace) -> bool {
    let (dx, dy, dz) = face.neighbor_offset();
    let neighbor_x = x as i32 + dx;
    let neighbor_y = y as i32 + dy;
    let neighbor_z = z as i32 + dz;

    if neighbor_x < 0
        || neighbor_x >= CHUNK_SIZE as i32
        || neighbor_y < 0
        || neighbor_y >= CHUNK_SIZE as i32
        || neighbor_z < 0
        || neighbor_z >= CHUNK_SIZE as i32
    {
        return true;
    }

    let neighbor_voxel = voxels.get(
        neighbor_x as usize,
        neighbor_y as usize,
        neighbor_z as usize,
    );
    !neighbor_voxel.is_solid()
}

fn add_face_to_mesh(
    positions: &mut Vec<[f32; 3]>,
    indices: &mut Vec<u32>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    pos: [f32; 3],
    face: CubeFace,
    voxel_type: VoxelType,
) {
    let base_index = positions.len() as u32;
    let vertices = face.vertices(pos);
    let normal = face.normal();
    let base_uvs = face.base_uvs();
    let texture_coords = get_texture_coords(voxel_type);

    positions.extend_from_slice(&vertices);

    for _ in 0..4 {
        normals.push(normal);
    }

    for &base_uv in &base_uvs {
        let atlas_uv = [
            texture_coords.0 + base_uv[0] * TEXTURE_SIZE,
            texture_coords.1 + base_uv[1] * TEXTURE_SIZE,
        ];
        uvs.push(atlas_uv);
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

fn get_texture_coords(voxel_type: VoxelType) -> (f32, f32) {
    match voxel_type {
        VoxelType::STONE => (0.0 * TEXTURE_SIZE, 0.0 * TEXTURE_SIZE),
        VoxelType::DIRT => (1.0 * TEXTURE_SIZE, 0.0 * TEXTURE_SIZE),
        VoxelType::AIR => (0.0, 0.0),
        _ => (15.0 * TEXTURE_SIZE, 15.0 * TEXTURE_SIZE),
    }
}
