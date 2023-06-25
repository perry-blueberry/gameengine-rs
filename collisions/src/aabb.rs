use crate::triangle_ray::Vertex;
use math::vector3::Vector3;

pub struct AABB {
    min: Vector3,
    max: Vector3,
}

impl AABB {
    pub fn new(mesh: &[Vertex]) -> Self {
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
        for &Vertex { position } in mesh {
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
