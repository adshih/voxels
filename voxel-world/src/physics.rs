use glam::IVec3;
use rapier3d::prelude::*;
use std::collections::HashMap;
use voxel_core::{
    VoxelBuffer,
    mesh::{Mesher, block::BlockMesher},
};

use crate::terrain::CHUNK_SIZE;

pub type BodyHandle = RigidBodyHandle;

pub struct Physics {
    // Config
    gravity: Vec3,
    integration_parameters: IntegrationParameters,

    // Data
    impulse_joint_set: ImpulseJointSet,
    multibody_joint_set: MultibodyJointSet,
    rigid_body_set: RigidBodySet,
    collider_set: ColliderSet,
    chunk_colliders: HashMap<IVec3, ColliderHandle>,

    // Machinery
    physics_pipeline: PhysicsPipeline,
    island_manager: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    ccd_solver: CCDSolver,

    // Callbacks
    physics_hooks: (),
    event_handler: (),
}

impl Physics {
    pub fn init() -> Self {
        Self::empty()
    }

    fn empty() -> Self {
        Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            chunk_colliders: HashMap::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.integration_parameters.dt = dt;
        self.physics_pipeline.step(
            self.gravity,
            &self.integration_parameters,
            &mut self.island_manager,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_body_set,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            &mut self.ccd_solver,
            &self.physics_hooks,
            &self.event_handler,
        );
    }

    pub fn set_force(&mut self, handle: BodyHandle, force: Vec3) {
        let body = &mut self.rigid_body_set[handle];
        body.reset_forces(true);
        body.add_force(force, true);
    }

    pub fn add_body(&mut self, pos: Vec3) -> BodyHandle {
        let body = RigidBodyBuilder::dynamic().translation(pos).build();
        let handle = self.rigid_body_set.insert(body);
        let collider = ColliderBuilder::ball(0.5).build();

        self.collider_set
            .insert_with_parent(collider, handle, &mut self.rigid_body_set);

        handle
    }

    pub fn position(&self, handle: BodyHandle) -> Vec3 {
        self.rigid_body_set[handle].translation()
    }

    pub fn remove_body(&mut self, handle: BodyHandle) {
        self.rigid_body_set.remove(
            handle,
            &mut self.island_manager,
            &mut self.collider_set,
            &mut self.impulse_joint_set,
            &mut self.multibody_joint_set,
            true, // also remove attached colliders
        );
    }

    pub fn add_chunk(&mut self, chunk_pos: IVec3, buffer: &VoxelBuffer) {
        if self.chunk_colliders.contains_key(&chunk_pos) {
            return;
        }

        let Some(mesh) = BlockMesher.generate(buffer) else {
            return;
        };

        let vertices: Vec<Vec3> = mesh
            .positions
            .iter()
            .copied()
            .map(Vec3::from_array)
            .collect();
        let indices: Vec<[u32; 3]> = mesh
            .indices
            .chunks_exact(3)
            .map(|t| [t[0], t[1], t[2]])
            .collect();

        let origin = (chunk_pos * CHUNK_SIZE.as_ivec3()).as_vec3();

        let collider = match ColliderBuilder::trimesh(vertices, indices) {
            Ok(builder) => builder.translation(origin).build(),
            Err(err) => {
                eprintln!("skipping trimesh collider for chunk {chunk_pos:?}: {err:?}");
                return;
            }
        };

        let handle = self.collider_set.insert(collider);
        self.chunk_colliders.insert(chunk_pos, handle);
    }

    pub fn remove_chunk(&mut self, chunk_pos: IVec3) {
        if let Some(handle) = self.chunk_colliders.remove(&chunk_pos) {
            self.collider_set.remove(
                handle,
                &mut self.island_manager,
                &mut self.rigid_body_set,
                true,
            );
        }
    }

    pub fn has_chunk(&self, chunk_pos: IVec3) -> bool {
        self.chunk_colliders.contains_key(&chunk_pos)
    }

    pub fn loaded_chunks(&self) -> impl Iterator<Item = IVec3> + '_ {
        self.chunk_colliders.keys().copied()
    }
}
