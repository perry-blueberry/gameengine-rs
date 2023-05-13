use super::{quaternion::Quaternion, vector3::Vector3};

#[derive(Debug, PartialEq, Clone)]
pub struct Transform {
    pub translation: Vector3,
    pub rotation: Quaternion,
    pub scale: Vector3,
}

impl Transform {
    pub fn combine(&self, other: &Self) -> Self {
        let scale = self.scale * other.scale;
        let rotation = other.rotation * self.rotation;
        let translation = self.translation + self.rotation * (self.scale * other.translation);
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn inverse(&self) -> Self {
        let rotation = self.rotation.inverse();
        let scale = self.scale.inverse();
        let translation = rotation * (scale * -self.translation);
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn mix(&self, other: &Self, t: f32) -> Self {
        let other_rotation = if self.rotation.dot(other.rotation) < 0.0 {
            -other.rotation
        } else {
            other.rotation
        };
        Transform {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.nlerp(other_rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }

    pub fn transform_point(&self, p: Vector3) -> Vector3 {
        self.translation + self.rotation * (self.scale * p)
    }

    pub fn transform_vector(&self, v: Vector3) -> Vector3 {
        self.rotation * (self.scale * v)
    }
}
