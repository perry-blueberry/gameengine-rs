use num_traits::Float;
use std::ops::{AddAssign, Mul};

use cgmath::Vector3;

pub trait AddScaledVector<F> {
    fn add_scaled_vector(&mut self, vector: &Vector3<F>, scale: F);
}

// impl<T: Float + Mul<T, Output = T> + AddAssign<<T as Mul>::Output>> AddScaledVector<T>

impl<F: Float + AddAssign> AddScaledVector<F> for Vector3<F> {
    fn add_scaled_vector(&mut self, vector: &Vector3<F>, scale: F) {
        self.x += vector.x * scale;
        self.y += vector.y * scale;
        self.z += vector.z * scale;
    }
}
