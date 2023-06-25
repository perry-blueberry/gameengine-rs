use num_traits::Zero;
use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

const EPSILON: f32 = 0.000001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vector3 {
    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Self) -> Self {
        Self {
            x: self.y * other.z - self.z * other.y,
            y: self.z * other.x - self.x * other.z,
            z: self.x * other.y - self.y * other.x,
        }
    }

    pub fn len_sq(&self) -> f32 {
        self.dot(*self)
    }

    pub fn len(&self) -> f32 {
        self.len_sq().sqrt()
    }

    pub fn normalize(&mut self) {
        let len = self.len();
        if len > EPSILON {
            let inv_len = 1.0 / len;
            *self *= inv_len;
        }
    }

    pub fn normalized(&self) -> Self {
        let mut clone = *self;
        clone.normalize();
        clone
    }

    pub fn angle_between(&self, other: Self) -> f32 {
        let len_product = self.len() * other.len();

        if len_product < EPSILON {
            return 0.0;
        }

        // Ensure the value is in the range for acos
        let cos_theta = (self.dot(other) / len_product).max(-1.0).min(1.0);

        cos_theta.acos()
    }

    pub fn project(&self, other: Self) -> Self {
        let len_sq = other.len_sq();
        if len_sq < EPSILON {
            return Self::zero();
        }
        let scale = self.dot(other) / len_sq;
        other * scale
    }

    pub fn reject(&self, other: Self) -> Self {
        *self - self.project(other)
    }

    pub fn lerp(&self, other: Self, t: f32) -> Self {
        Self {
            x: self.x + (other.x - self.x) * t,
            y: self.y + (other.y - self.y) * t,
            z: self.z + (other.z - self.z) * t,
        }
    }

    pub fn nlerp(&self, other: Self, t: f32) -> Self {
        self.lerp(other, t).normalized()
    }

    pub fn inverse(&self) -> Self {
        let x = if self.x.abs() < EPSILON {
            1.0
        } else {
            1.0 / self.x
        };
        let y = if self.y.abs() < EPSILON {
            1.0
        } else {
            1.0 / self.y
        };
        let z = if self.z.abs() < EPSILON {
            1.0
        } else {
            1.0 / self.z
        };
        Self { x, y, z }
    }

    pub fn magnitude(&self) -> f32 {
        self.len()
    }

    pub fn add_scaled_vector(&mut self, vector: Vector3, scale: f32) {
        self.x += vector.x * scale;
        self.y += vector.y * scale;
        self.z += vector.z * scale;
    }

    pub fn up() -> Self {
        Self {
            x: 0.0,
            y: 1.0,
            z: 0.0,
        }
    }

    pub fn right() -> Self {
        Self {
            x: 1.0,
            y: 0.0,
            z: 0.0,
        }
    }

    /* pub fn reflect(&self, normal: Self) -> Self {
        *self - *normal * 2.0 * self.dot(normal)
    } */
}

impl Add for Vector3 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl Sub for Vector3 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl Mul for Vector3 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: self.x * other.x,
            y: self.y * other.y,
            z: self.z * other.z,
        }
    }
}

impl Mul<f32> for Vector3 {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
        }
    }
}

impl AddAssign for Vector3 {
    fn add_assign(&mut self, other: Self) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl SubAssign for Vector3 {
    fn sub_assign(&mut self, other: Self) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

impl MulAssign for Vector3 {
    fn mul_assign(&mut self, other: Self) {
        self.x *= other.x;
        self.y *= other.y;
        self.z *= other.z;
    }
}

impl MulAssign<f32> for Vector3 {
    fn mul_assign(&mut self, scalar: f32) {
        self.x *= scalar;
        self.y *= scalar;
        self.z *= scalar;
    }
}

impl Zero for Vector3 {
    fn zero() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }

    fn is_zero(&self) -> bool {
        self.x == 0.0 && self.y == 0.0 && self.z == 0.0
    }
}

impl From<[f32; 3]> for Vector3 {
    fn from(value: [f32; 3]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
        }
    }
}

impl Into<[f32; 3]> for Vector3 {
    fn into(self) -> [f32; 3] {
        [self.x, self.y, self.z]
    }
}

impl Neg for Vector3 {
    type Output = Vector3;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }
}

impl Default for Vector3 {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<glam::Vec3> for Vector3 {
    fn from(value: glam::Vec3) -> Self {
        Self {
            x: value.x,
            y: value.y,
            z: value.z,
        }
    }
}
