use glam::{Quat, Vec3};

use crate::math::{glam_transform::FromTo, glam_transform::Transform};

#[derive(Debug)]
pub struct FabrikSolver {
    ik_chain: Vec<Transform>,
    pub num_steps: u8,
    pub threshold: f32,
    world_chain: Vec<Vec3>,
    lengths: Vec<f32>,
}

impl FabrikSolver {
    pub fn new() -> Self {
        Self::new_with_args(15, 0.00001)
    }

    pub fn new_with_args(num_steps: u8, threshold: f32) -> Self {
        Self {
            ik_chain: vec![],
            num_steps,
            threshold,
            world_chain: vec![],
            lengths: vec![],
        }
    }

    pub fn resize(&mut self, len: usize) {
        self.ik_chain.resize(len, Transform::default());
        self.world_chain.resize(len, Vec3::default());
        self.lengths.resize(len, 0.0);
    }

    pub fn solve(&mut self, target: &Transform) -> bool {
        if self.len() == 0 {
            return false;
        }

        let threshold_squared = self.threshold * self.threshold;
        self.ik_chain_to_world();
        let goal = target.translation;
        let base = self.world_chain[0];

        for _ in 0..self.num_steps {
            let effector = self.world_chain.last().unwrap();
            if goal.distance_squared(*effector) < threshold_squared {
                self.world_to_ik_chain();
                return true;
            }

            self.iterate_backward(goal);
            self.iterate_forward(base);
        }

        self.world_to_ik_chain();
        let effector = self.global_transform(self.len() - 1).translation;
        if goal.distance_squared(effector) < threshold_squared {
            return true;
        }

        false
    }

    pub fn len(&self) -> usize {
        self.ik_chain.len()
    }

    pub fn local_transform(&self, index: usize) -> &Transform {
        &self.ik_chain[index]
    }

    pub fn set_local_transform(&mut self, index: usize, t: Transform) {
        self.ik_chain[index] = t;
    }

    pub fn global_transform(&self, index: usize) -> Transform {
        let mut world = self.ik_chain[index].clone();
        for transform in self.ik_chain[0..index].iter().rev() {
            world = transform.combine(&world);
        }
        world.to_owned()
    }

    fn ik_chain_to_world(&mut self) {
        for i in 0..self.len() {
            let world = self.global_transform(i);
            self.world_chain[i] = world.translation;
            if i >= 1 {
                let prev = self.world_chain[i - 1];
                self.lengths[i] = (world.translation - prev).length();
            }
        }

        if self.len() > 0 {
            self.lengths[0] = 0.0;
        }
    }

    fn world_to_ik_chain(&mut self) {
        if self.len() == 0 {
            return;
        }

        for i in 0..self.len() - 1 {
            let world = self.global_transform(i);
            let next = self.global_transform(i + 1);
            let position = world.translation;
            let inv_rot = world.rotation.inverse();

            let to_next = inv_rot * (next.translation - position);
            let to_desired = inv_rot * (self.world_chain[i + 1] - position);

            let delta = Quat::from_to(to_next, to_desired);
            self.ik_chain[i].rotation = self.ik_chain[i].rotation * delta;
        }
    }

    fn iterate_backward(&mut self, goal: Vec3) {
        let len = self.len();
        if len == 0 {
            return;
        }
        self.world_chain[len - 1] = goal;

        for i in (0..len - 1).rev() {
            let direction = (self.world_chain[i] - self.world_chain[i + 1]).normalize();
            let offset = direction * self.lengths[i + 1];
            self.world_chain[i] = self.world_chain[i + 1] + offset;
        }
    }

    fn iterate_forward(&mut self, base: Vec3) {
        if self.len() == 0 {
            return;
        }
        self.world_chain[0] = base;

        for i in 1..self.len() {
            let direction = (self.world_chain[i] - self.world_chain[i - 1]).normalize();
            let offset = direction * self.lengths[i];
            self.world_chain[i] = self.world_chain[i - 1] + offset;
        }
    }
}
