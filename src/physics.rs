//! Deterministic Rapier2D pendulum. Physics never touches the DOM or the UI.

use rapier2d::prelude::*;

use crate::domain::PendulumBob;

/// Fixed integration step: every run with the same inputs gives the same result.
pub const FIXED_DT: f32 = 1.0 / 120.0;

pub struct Simulation {
    pipeline: PhysicsPipeline,
    bodies: RigidBodySet,
    colliders: ColliderSet,
    joints: ImpulseJointSet,
    multibodies: MultibodyJointSet,
    islands: IslandManager,
    broad_phase: DefaultBroadPhase,
    narrow_phase: NarrowPhase,
    ccd: CCDSolver,
    integration: IntegrationParameters,
    bob_handle: RigidBodyHandle,
    threshold_deg: f32,
    was_inside: bool,
    pub angle_deg: f32,
    pub swings: u32,
    pub elapsed: f32,
}

impl Simulation {
    pub fn new(length: f32, release_deg: f32, threshold_deg: f32, bob: &PendulumBob) -> Self {
        let mut bodies = RigidBodySet::new();
        let mut colliders = ColliderSet::new();
        let mut joints = ImpulseJointSet::new();

        let anchor = bodies.insert(RigidBodyBuilder::fixed().build());
        let angle = release_deg.to_radians();
        let bob_handle = bodies.insert(
            RigidBodyBuilder::dynamic()
                .translation(vector![length * angle.sin(), length * angle.cos()])
                .build(),
        );

        let area = std::f32::consts::PI * bob.radius * bob.radius;
        colliders.insert_with_parent(
            ColliderBuilder::ball(bob.radius)
                .density(bob.mass / area)
                .build(),
            bob_handle,
            &mut bodies,
        );

        // The pivot is a point on the (fixed) anchor body. On the bob body it
        // is offset by the release position, so the joint starts satisfied and
        // the bob centre can trace a circle of `length` around the pivot.
        let joint = RevoluteJointBuilder::new()
            .local_anchor1(point![0.0, 0.0])
            .local_anchor2(point![-length * angle.sin(), -length * angle.cos()])
            .build();
        joints.insert(anchor, bob_handle, joint, true);

        let mut integration = IntegrationParameters::default();
        integration.dt = FIXED_DT;

        Self {
            pipeline: PhysicsPipeline::new(),
            bodies,
            colliders,
            joints,
            multibodies: MultibodyJointSet::new(),
            islands: IslandManager::new(),
            broad_phase: DefaultBroadPhase::default(),
            narrow_phase: NarrowPhase::new(),
            ccd: CCDSolver::new(),
            integration,
            bob_handle,
            threshold_deg,
            was_inside: release_deg.abs() < threshold_deg,
            angle_deg: release_deg,
            swings: 0,
            elapsed: 0.0,
        }
    }

    /// Advance one fixed step. Returns true when the bob crossed a threshold.
    pub fn step(&mut self) -> bool {
        let gravity = vector![0.0, 9.81];
        self.pipeline.step(
            &gravity,
            &self.integration,
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.bodies,
            &mut self.colliders,
            &mut self.joints,
            &mut self.multibodies,
            &mut self.ccd,
            None,
            &(),
            &(),
        );
        self.elapsed += FIXED_DT;

        let Some(body) = self.bodies.get(self.bob_handle) else {
            return false;
        };
        let position = body.translation();
        self.angle_deg = position.x.atan2(position.y).to_degrees();

        // A "swing" is an outward crossing of the threshold on either side.
        let outside = self.angle_deg.abs() >= self.threshold_deg;
        let crossed = self.was_inside && outside;
        if crossed {
            self.swings += 1;
        }
        self.was_inside = !outside;
        crossed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pendulum_swings_and_stays_finite() {
        let bob = &crate::domain::BOB_PRESETS[0];
        let mut sim = Simulation::new(0.5, 30.0, 5.0, bob);
        for _ in 0..1200 {
            sim.step();
        }
        assert!(
            sim.swings >= 4,
            "expected several swings, got {}",
            sim.swings
        );
        assert!(sim.angle_deg.is_finite());
        assert!(sim.angle_deg.abs() <= 31.0);
        assert!((sim.elapsed - 10.0).abs() < 0.01);
    }

    #[test]
    fn same_inputs_give_same_output() {
        let bob = &crate::domain::BOB_PRESETS[1];
        let mut a = Simulation::new(0.8, 45.0, 10.0, bob);
        let mut b = Simulation::new(0.8, 45.0, 10.0, bob);
        for _ in 0..600 {
            a.step();
            b.step();
        }
        assert_eq!(a.swings, b.swings);
        assert_eq!(a.angle_deg.to_bits(), b.angle_deg.to_bits());
    }
}
