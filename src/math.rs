use cgmath::{Quaternion, Vector3};

pub trait AddScaledVector<f32> {
    fn add_scaled_vector(&mut self, vector: &Vector3<f32>, scale: f32);
}

impl AddScaledVector<f32> for Vector3<f32> {
    fn add_scaled_vector(&mut self, vector: &Vector3<f32>, scale: f32) {
        self.x += vector.x * scale;
        self.y += vector.y * scale;
        self.z += vector.z * scale;
    }
}

pub fn mix(from: &Quaternion<f32>, to: &Quaternion<f32>, t: f32) -> Quaternion<f32> {
    from * (1.0 - t) + to * t
}
