pub mod mesh;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Debug)]
pub struct Voxel(pub u16);

impl Voxel {
    pub const EMPTY: Self = Self(0);
    pub const DIRT: Self = Self(1);
    pub const STONE: Self = Self(2);

    pub fn is_empty(&self) -> bool {
        *self == Self::EMPTY
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct VoxelBuffer {
    pub size: [u32; 3],
    pub voxels: Vec<Voxel>,
}

impl VoxelBuffer {
    pub fn new(size: [u32; 3]) -> Self {
        let volume = (size[0] * size[1] * size[2]) as usize;
        Self {
            size,
            voxels: vec![Voxel::EMPTY; volume],
        }
    }

    pub fn get(&self, pos: [u32; 3]) -> Voxel {
        let i = self.index(pos);
        self.voxels[i]
    }

    pub fn set(&mut self, pos: [u32; 3], voxel: Voxel) {
        let i = self.index(pos);
        self.voxels[i] = voxel;
    }

    pub fn is_all_empty(&self) -> bool {
        self.voxels.iter().all(|v| v.is_empty())
    }

    fn index(&self, pos: [u32; 3]) -> usize {
        (pos[0] + pos[1] * self.size[0] + pos[2] * self.size[0] * self.size[1]) as usize
    }
}
