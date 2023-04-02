use cgmath::{Quaternion, Vector3};

use super::array_type::ArrayType;

/// Used to store keyframes in a Track
pub(crate) struct Frame<A: ArrayType> {
    pub(crate) value: A::Slice,
    pub(crate) in_tangent: A::Slice,
    pub(crate) out_tangent: A::Slice,
    pub(crate) time: f32,
}

type ScalarFrame = Frame<f32>;
type Vector3Frame = Frame<Vector3<f32>>;
type QuatFrame = Frame<Quaternion<f32>>;
