use rapier3d::prelude::*;

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
        let mut physics = Self {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            integration_parameters: IntegrationParameters::default(),
            impulse_joint_set: ImpulseJointSet::new(),
            multibody_joint_set: MultibodyJointSet::new(),
            rigid_body_set: RigidBodySet::new(),
            collider_set: ColliderSet::new(),
            physics_pipeline: PhysicsPipeline::new(),
            island_manager: IslandManager::new(),
            broad_phase: DefaultBroadPhase::new(),
            narrow_phase: NarrowPhase::new(),
            ccd_solver: CCDSolver::new(),
            physics_hooks: (),
            event_handler: (),
        };

        // TODO: remove this and replace with real terrain colliders
        let floor = ColliderBuilder::cuboid(1000.0, 50.0, 1000.0).build();
        physics.collider_set.insert(floor);

        physics
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
}
