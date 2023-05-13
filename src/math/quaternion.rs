use std::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use super::vector3::Vector3;

const EPSILON: f32 = 0.000001;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

impl Quaternion {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }

    pub fn new_vs(v: Vector3, s: f32) -> Self {
        Self {
            x: v.x,
            y: v.y,
            z: v.z,
            w: s,
        }
    }

    pub fn from_axis_angle(axis: Vector3, angle: f32) -> Self {
        let s = (angle.to_radians() * 0.5).sin();
        Self {
            x: axis.x * s,
            y: axis.y * s,
            z: axis.z * s,
            w: (angle.to_radians() * 0.5).cos(),
        }
    }

    pub fn v(&self) -> Vector3 {
        Vector3 {
            x: self.x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn s(&self) -> f32 {
        self.w
    }

    pub fn set_v(&mut self, v: Vector3) {
        self.x = v.x;
        self.y = v.y;
        self.z = v.z;
    }

    pub fn set_s(&mut self, s: f32) {
        self.w = s;
    }

    pub fn axis(&self) -> Vector3 {
        self.v().normalized()
    }

    pub fn angle(&self) -> f32 {
        2.0 * self.w.acos()
    }

    pub fn dot(&self, other: Self) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z + self.w * other.w
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

    pub fn conjugate(&self) -> Self {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: self.w,
        }
    }

    pub fn inverse(&self) -> Self {
        let len_sq = self.len_sq();
        if len_sq < EPSILON {
            return Quaternion::default();
        }
        let recip = 1.0 / len_sq;
        self.conjugate() * recip
    }

    pub fn mix(&self, other: Self, t: f32) -> Self {
        *self * (1.0 - t) + other * t
    }

    pub fn nlerp(&self, other: Self, t: f32) -> Self {
        (*self + (other - *self) * t).normalized()
    }
    pub fn look_rotation(direction: Vector3, up: Vector3) -> Self {
        let f = direction.normalized();
        let u = up.normalized();
        let r = u.cross(f);
        let u = f.cross(r);

        let f2d = Quaternion::from_to(
            Vector3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            },
            f,
        );
        let object_up = f2d * Vector3::up();
        let u2u = Quaternion::from_to(object_up, u);

        (f2d * u2u).normalized()
    }

    pub fn from_to(from: Vector3, to: Vector3) -> Self {
        let from = from.normalized();
        let to = to.normalized();

        if from == to {
            return Quaternion::default();
        }

        if from == to * -1.0 {
            let ortho = if from.x < from.y && from.x < from.z {
                Vector3::right()
            } else if from.y < from.z {
                Vector3::up()
            } else {
                Vector3 {
                    x: 0.0,
                    y: 0.0,
                    z: 1.0,
                }
            };
            let axis = from.cross(ortho).normalized();
            return Quaternion::new_vs(axis, 0.0);
        }

        let half = (from + to).normalized();
        let axis = from.cross(half);
        Quaternion::new_vs(axis, from.dot(half))
    }
}

impl Add for Quaternion {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
            w: self.w + other.w,
        }
    }
}

impl Sub for Quaternion {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
            w: self.w - other.w,
        }
    }
}

impl Mul for Quaternion {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        Self {
            x: other.x * self.w + other.y * self.z - other.z * self.y + other.w * self.x,
            y: -other.x * self.z + other.y * self.w + other.z * self.x + other.w * self.y,
            z: other.x * self.y - other.y * self.x + other.z * self.w + other.w * self.z,
            w: -other.x * self.x - other.y * self.y - other.z * self.z + other.w * self.w,
        }
    }
}

impl Mul<Vector3> for Quaternion {
    type Output = Vector3;

    fn mul(self, other: Vector3) -> Self::Output {
        self.v() * 2.0 * self.v().dot(other)
            + other * (self.s() * self.s() - self.v().len_sq())
            + self.v().cross(other) * 2.0 * self.s()
    }
}

impl Mul<f32> for Quaternion {
    type Output = Self;

    fn mul(self, scalar: f32) -> Self {
        Self {
            x: self.x * scalar,
            y: self.y * scalar,
            z: self.z * scalar,
            w: self.w * scalar,
        }
    }
}

impl AddAssign for Quaternion {
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl SubAssign for Quaternion {
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl MulAssign<f32> for Quaternion {
    fn mul_assign(&mut self, scalar: f32) {
        *self = *self * scalar;
    }
}

impl Default for Quaternion {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            w: 1.0,
        }
    }
}

impl Neg for Quaternion {
    type Output = Quaternion;

    fn neg(self) -> Self::Output {
        Self {
            x: -self.x,
            y: -self.y,
            z: -self.z,
            w: -self.w,
        }
    }
}

impl From<[f32; 4]> for Quaternion {
    fn from(value: [f32; 4]) -> Self {
        Self {
            x: value[0],
            y: value[1],
            z: value[2],
            w: value[3],
        }
    }
}

/* #[derive(Clone, Copy)]
struct XYZW {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Clone, Copy)]
struct VS {
    pub v: Vector3,
    pub s: f32,
}

union Q {
    xyzw: XYZW,
    vs: VS,
}

struct Quaternion {
    q: Q,
}

impl Q {
    pub fn new_xyzw(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self {
            xyzw: XYZW { x, y, z, w },
        }
    }

    pub fn new_vs(v: Vector3, s: f32) -> Self {
        Self { vs: VS { v, s } }
    }
}

impl Quaternion {
    pub fn new_vs(v: Vector3, s: f32) -> Self {
        Self { q: Q::new_vs(v, s) }
    }

    pub fn get_angle(&self) -> f32 {
        2.0 * self.q.vs.s.acos()
    }
} */

/* impl One for Quaternion {
    fn one() -> Self {
        Quaternion::
    }
} */
