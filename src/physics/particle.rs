use std::{
    fmt::Debug,
    ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign},
};

use cgmath::Vector3;
use num_traits::Float;

use crate::math::AddScaledVector;

pub struct Particle {
    pub(crate) position: Vector3<f32>,
    pub(crate) velocity: Vector3<f32>,
    pub(crate) force_accum: Vector3<f32>,
    pub(crate) acceleration: Vector3<f32>,
    pub(crate) damping: F,
    pub(crate) inverse_mass: F,
}

impl Particle {
    pub fn integrate(&mut self, delta: F) {
        assert!(delta > F::zero());

        self.position.add_scaled_vector(&self.velocity, delta);

        let mut resulting_acc = self.acceleration;
        resulting_acc.add_scaled_vector(&self.force_accum, self.inverse_mass);

        self.velocity.add_scaled_vector(&resulting_acc, delta);

        self.velocity *= Float::powf(self.damping, delta);
    }
}
