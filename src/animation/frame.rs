/// Used to store keyframes in a Track
pub struct Frame<const N: usize> {
    pub(crate) value: [f32; N],
    pub(crate) in_tangent: [f32; N],
    pub(crate) out_tangent: [f32; N],
    pub(crate) time: f32,
}

type ScalarFrame = Frame<1>;
type Vector3Frame = Frame<3>;
type QuatFrame = Frame<4>;
