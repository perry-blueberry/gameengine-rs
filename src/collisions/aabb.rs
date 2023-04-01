use std::{
    fmt::Debug,
    ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign},
};

use cgmath::Vector3;
use num_traits::Float;

use crate::model;

pub struct AABB<F: Float + AddAssign + MulAssign + SubAssign + DivAssign + RemAssign + Debug> {
    min: Vector3<F>,
    max: Vector3<F>,
}

impl<F: Float + AddAssign + MulAssign + SubAssign + DivAssign + RemAssign + Debug> AABB<F> {
    pub fn new(mesh: &model::Mesh<F>) -> Self {
        let mut min = Vector3::new(F::infinity(), F::infinity(), F::infinity());
        let mut max = Vector3::new(F::neg_infinity(), F::neg_infinity(), F::neg_infinity());
        for model::ModelVertex {
            position,
            tex_coords,
            normal,
        } in mesh.model_vertices
        {
            min.x = F::min(min.x, position[0]);
            min.y = F::min(min.y, position[1]);
            min.z = F::min(min.z, position[2]);
            max.x = F::min(max.x, position[0]);
            max.y = F::min(max.y, position[1]);
            max.z = F::min(max.z, position[2]);
        }

        Self { min, max }
    }
}
