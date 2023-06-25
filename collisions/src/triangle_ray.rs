use glam::Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

pub struct Vertex {
    pub position: [f32; 3],
}

impl Ray {
    pub fn new(origin: Vec3) -> Self {
        Self {
            origin,
            direction: Vec3::new(0.0, -1.0, 0.0),
        }
    }
    pub fn cast(&self, triangle: &Triangle) -> Option<Vec3> {
        const EPSILON: f32 = 0.0000001;
        let (v0, v1, v2) = (triangle.v0, triangle.v1, triangle.v2);
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let ray_vector = self.direction;
        let origin = self.origin;
        let h = ray_vector.cross(edge2);
        let a = edge1.dot(h);
        if a.abs() < EPSILON {
            return None;
        }
        let f = 1.0 / a;
        let s = origin - v0;
        let u = f * s.dot(h);
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let q = s.cross(edge1);
        let v = f * ray_vector.dot(q);
        if v < 0.0 || u + v > 1.0 {
            return None;
        }
        let t = f * edge2.dot(q);
        if t <= EPSILON {
            return None;
        }
        Some(origin + ray_vector * t)
    }
}

pub struct Triangle {
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    normal: Vec3,
}

impl Triangle {
    pub fn new(v0: Vec3, v1: Vec3, v2: Vec3) -> Self {
        Self {
            v0,
            v1,
            v2,
            normal: Vec3::cross(v1 - v0, v2 - v0).normalize(),
        }
    }
}

pub fn mesh_to_triangles(vertices: &[Vertex], indices: &[u32]) -> Vec<Triangle> {
    let mut result = Vec::with_capacity(indices.len() / 3);
    for i in (0..indices.len()).step_by(3) {
        result.push(Triangle::new(
            vertices[indices[i] as usize].position.into(),
            vertices[indices[i + 1] as usize].position.into(),
            vertices[indices[i + 2] as usize].position.into(),
        ));
    }
    result
}
