use super::vector3::Vector3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector4<T> {
    pub x: T,
    pub y: T,
    pub z: T,
    pub w: T,
}

impl Vector4<f32> {
    pub fn new(v: Vector3, w: f32) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w,
        }
    }
}

impl Vector4<f32> {
    pub fn truncate(&self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }
}
