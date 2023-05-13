use crate::math::quaternion::Quaternion;
use crate::math::vector3::Vector3;

use super::track::DefaultConstructible;

use super::array_type::ArrayType;

/// Used to store keyframes in a Track
#[derive(Debug, Clone)]
pub struct Frame<A: ArrayType> {
    pub value: A::Slice,
    pub in_tangent: A::Slice,
    pub out_tangent: A::Slice,
    pub time: f32,
}

pub type ScalarFrame = Frame<f32>;
pub type Vector3Frame = Frame<Vector3>;
pub type QuatFrame = Frame<Quaternion>;

impl<A: ArrayType> Frame<A> {
    pub fn new_simple(time: f32, value: A) -> Frame<A>
    where
        A: DefaultConstructible,
    {
        Self::new(time, A::default(), A::default(), value)
    }

    pub fn new(time: f32, in_tangent: A, out_tangent: A, value: A) -> Frame<A> {
        Self {
            value: value.to_slice(),
            in_tangent: in_tangent.to_slice(),
            out_tangent: out_tangent.to_slice(),
            time,
        }
    }
}
