use glam::UVec3;
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
    pub size: UVec3,
    pub voxels: Vec<Voxel>,
}

impl VoxelBuffer {
    pub fn new(size: UVec3) -> Self {
        let volume = (size.x * size.y * size.z) as usize;
        Self {
            size,
            voxels: vec![Voxel::EMPTY; volume],
        }
    }

    pub fn get(&self, pos: UVec3) -> Voxel {
        let i = self.index(pos);
        self.voxels[i]
    }

    pub fn set(&mut self, pos: UVec3, voxel: Voxel) {
        let i = self.index(pos);
        self.voxels[i] = voxel;
    }

    fn index(&self, pos: UVec3) -> usize {
        (pos.x + pos.y * self.size.x + pos.z * self.size.x * self.size.y) as usize
    }
}
