use glam::{Mat4, Quat, Vec3};

#[derive(Debug, PartialEq, Clone)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl Transform {
    pub fn new(translation: Vec3, rotation: Quat, scale: Vec3) -> Self {
        Self {
            translation,
            rotation,
            scale,
        }
    }

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

impl Default for Transform {
    fn default() -> Self {
        Self {
            translation: Default::default(),
            rotation: Default::default(),
            scale: Vec3::ONE,
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

pub trait FromTo {
    fn from_to(from: Vec3, to: Vec3) -> Quat;
}

impl FromTo for Quat {
    fn from_to(from: Vec3, to: Vec3) -> Quat {
        let from = from.normalize();
        let to = to.normalize();

        if from == to {
            return Quat::default();
        }

        if from == to * -1.0 {
            let ortho = if from.x < from.y && from.x < from.z {
                Vec3::X
            } else if from.y < from.z {
                Vec3::Y
            } else {
                Vec3::Z
            };
            let axis = from.cross(ortho).normalize();
            return Quat::from_axis_angle(axis, 0.0);
        }

        let half = (from + to).normalize();
        let axis = from.cross(half);
        Quat::from_xyzw(axis.x, axis.y, axis.z, from.dot(half))
        /* Quat::from_axis_angle(axis, from.dot(half)) */
    }
}

pub trait LookRotation {
    fn look_rotation(direction: Vec3, up: Vec3) -> Self;
}
impl LookRotation for Quat {
    fn look_rotation(direction: Vec3, up: Vec3) -> Self {
        let f = direction.normalize();
        let u = up.normalize();
        let r = u.cross(f);
        let u = f.cross(r);

        let f2d = Quat::from_rotation_arc(Vec3::Z, f);
        let object_up = f2d * Vec3::Y;
        let u2u = Quat::from_rotation_arc(object_up, u);

        /* HERE? */
        (u2u * f2d).normalize()
    }
}
