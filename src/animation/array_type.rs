use std::fmt::Debug;

use crate::math::{quaternion::Quaternion, vector3::Vector3};

pub trait ArrayType {
    const LENGTH: usize;
    type Slice: AsRef<[f32]> + AsMut<[f32]> + Debug + Clone;

    fn from_slice(array: &Self::Slice) -> Self;
    fn to_slice(&self) -> Self::Slice;
}

impl ArrayType for f32 {
    const LENGTH: usize = 1;
    type Slice = [f32; 1];

    fn from_slice(array: &Self::Slice) -> Self {
        array[0]
    }

    fn to_slice(&self) -> Self::Slice {
        [*self]
    }
}

impl ArrayType for Vector3 {
    const LENGTH: usize = 3;
    type Slice = [f32; 3];

    fn from_slice(array: &Self::Slice) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
        }
    }

    fn to_slice(&self) -> Self::Slice {
        [self.x, self.y, self.z]
    }
}

impl ArrayType for Quaternion {
    const LENGTH: usize = 4;
    type Slice = [f32; 4];

    fn from_slice(array: &Self::Slice) -> Self {
        Self {
            x: array[0],
            y: array[1],
            z: array[2],
            w: array[3],
        }
    }

    fn to_slice(&self) -> Self::Slice {
        [self.x, self.y, self.z, self.w]
    }
}
