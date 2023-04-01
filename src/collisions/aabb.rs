use std::{
    fmt::Debug,
    ops::{AddAssign, DivAssign, MulAssign, RemAssign, SubAssign},
};

use cgmath::Vector3;

use crate::model;

pub struct AABB {
    min: Vector3<f32>,
    max: Vector3<f32>,
}

impl AABB {
    pub fn new(mesh: &model::Mesh) -> Self {
        let mut min = Vector3::new(f32::INFINITY, f32::INFINITY, f32::INFINITY);
        let mut max = Vector3::new(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY);
        for &model::ModelVertex {
            position,
            tex_coords: _,
            normal: _,
        } in &mesh.model_vertices
        {
            min.x = min.x.min(position[0]);
            min.y = min.y.min(position[1]);
            min.z = min.z.min(position[2]);
            max.x = max.x.max(position[0]);
            max.y = max.y.max(position[1]);
            max.z = max.z.max(position[2]);
        }

        Self { min, max }
    }
}
