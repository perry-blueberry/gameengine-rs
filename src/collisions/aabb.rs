use crate::{
    math::vector3::Vector3,
    rendering::model::{self, ModelVertex},
};

pub struct AABB {
    min: Vector3,
    max: Vector3,
}

impl AABB {
    pub fn new(mesh: &model::Mesh<ModelVertex>) -> Self {
        let mut min = Vector3 {
            x: f32::INFINITY,
            y: f32::INFINITY,
            z: f32::INFINITY,
        };
        let mut max = Vector3 {
            x: f32::NEG_INFINITY,
            y: f32::NEG_INFINITY,
            z: f32::NEG_INFINITY,
        };
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
