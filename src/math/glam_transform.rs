use glam::{Mat4, Quat, Vec3};

#[derive(Debug, PartialEq, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn combine(&self, other: &Self) -> Self {
        let scale = self.scale * other.scale;
        let rotation = self.rotation * other.rotation;
        let translation = self.translation + self.rotation * (self.scale * other.translation);
        Self {
            translation,
            rotation,
            scale,
        }
    }

    pub fn inverse(&self) -> Self {
        let rotation = self.rotation.inverse();
        let scale = 1.0 / self.scale;
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
            rotation: self.rotation.lerp(other_rotation, t),
            scale: self.scale.lerp(other.scale, t),
        }
    }
}

impl Into<Mat4> for Transform {
    fn into(self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

impl From<Mat4> for Transform {
    fn from(value: Mat4) -> Self {
        let (scale, rotation, translation) = value.to_scale_rotation_translation();
        Self {
            translation,
            rotation,
            scale,
        }
    }
}
