use std::ops::Neg;

use cgmath::{InnerSpace, Quaternion, Vector3, VectorSpace};

use crate::math::mix;

pub(crate) trait Neighborhood {
    fn neighborhood(&self, other: &mut Self);
}

impl Neighborhood for f32 {
    fn neighborhood(&self, _other: &mut Self) {}
}

impl Neighborhood for Vector3<f32> {
    fn neighborhood(&self, _other: &mut Self) {}
}

impl Neighborhood for Quaternion<f32> {
    fn neighborhood(&self, other: &mut Self) {
        if self.dot(*other) < 0.0 {
            *other = other.neg();
        }
    }
}

pub(crate) trait AdjustHermiteResult {
    fn adjust_hermite_result(&self) -> Self;
}

impl AdjustHermiteResult for f32 {
    fn adjust_hermite_result(&self) -> Self {
        *self
    }
}

impl AdjustHermiteResult for Vector3<f32> {
    fn adjust_hermite_result(&self) -> Self {
        *self
    }
}

impl AdjustHermiteResult for Quaternion<f32> {
    fn adjust_hermite_result(&self) -> Self {
        self.normalize()
    }
}

pub(crate) trait Interpolate {
    fn interpolate(&self, other: &Self, t: f32) -> Self;
}

impl Interpolate for f32 {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self + (other - self) * t
    }
}

impl Interpolate for Vector3<f32> {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        self.lerp(*other, t)
    }
}

impl Interpolate for Quaternion<f32> {
    fn interpolate(&self, other: &Self, t: f32) -> Self {
        let result = if
        /*neighborhood */
        self.dot(*other) < 0.0 {
            mix(self, &-other, t)
        } else {
            mix(self, other, t)
        };
        result.normalize()
    }
}
