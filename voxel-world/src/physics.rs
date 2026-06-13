use rapier3d::prelude::*;

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
        Self {
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
        }
    }

    pub fn step(&mut self) {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn body_falls_and_rests_on_floor() {
        let mut physics = Physics::init();

        let floor = ColliderBuilder::cuboid(50.0, 0.5, 50.0).build();
        physics.collider_set.insert(floor);

        let body = RigidBodyBuilder::dynamic()
            .translation(Vec3::new(0.0, 10.0, 0.0))
            .build();
        let handle = physics.rigid_body_set.insert(body);

        let ball = ColliderBuilder::ball(0.5).build();
        physics
            .collider_set
            .insert_with_parent(ball, handle, &mut physics.rigid_body_set);

        for _ in 0..180 {
            physics.step();
        }

        let y = physics.rigid_body_set[handle].translation().y;
        println!("rested at y = {y}");
        assert!(y < 10.0, "should have fallen");
        assert!(y > 0.0, "should be above the floor");
    }

    #[test]
    fn force_cancels_gravity_and_hovers() {
        let mut physics = Physics::init();

        let body = RigidBodyBuilder::dynamic().translation(Vec3::new(0.0, 10.0, 0.0)).build();
        let handle = physics.rigid_body_set.insert(body);
        physics.collider_set.insert_with_parent(ColliderBuilder::ball(0.5).build(), handle, &mut physics.rigid_body_set);

        for _ in 0..180 {
            let body = &mut physics.rigid_body_set[handle];
            body.reset_forces(false);
            let weight = body.mass() * 9.81;
            body.add_force(Vec3::new(0.0, weight, 0.0), true);
            physics.step();
        }

        let y = physics.rigid_body_set[handle].translation().y;
        println!("hovered at y = {y}");
        assert!((y - 10.0).abs() < 0.1, "should still be -10, got {y}");
    }
}
