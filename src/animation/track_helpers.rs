use std::ops::Neg;

use crate::math::{quaternion::Quaternion, vector3::Vector3};

pub trait Neighborhood {
    fn neighborhood(&self, other: &mut Self);
}

impl Neighborhood for f32 {
    fn neighborhood(&self, _other: &mut Self) {}
}

impl Neighborhood for Vector3 {
    fn neighborhood(&self, _other: &mut Self) {}
}

impl Neighborhood for Quaternion {
    fn neighborhood(&self, other: &mut Self) {
        if self.dot(*other) < 0.0 {
            *other = other.neg();
        }
    }
}

pub trait AdjustHermiteResult {
    fn adjust_hermite_result(&self) -> Self;
}

impl AdjustHermiteResult for f32 {
    fn adjust_hermite_result(&self) -> Self {
        *self
    }
}

impl AdjustHermiteResult for Vector3 {
    fn adjust_hermite_result(&self) -> Self {
        *self
    }
}

impl AdjustHermiteResult for Quaternion {
    fn adjust_hermite_result(&self) -> Self {
        self.normalized()
    }
}

pub trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for Vector3 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolate for Quaternion {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let result = if
        /*neighborhood */
        self.dot(*other) < 0.0 {
            self.mix(-*other, t)
        } else {
            self.mix(*other, t)
        };
        result.normalized()
    }
}
