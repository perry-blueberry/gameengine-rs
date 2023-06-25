use std::ops::{Add, Mul, Sub};

use bytemuck::{Pod, Zeroable};
use num_traits::Zero;

use super::{
    matrix3::Matrix3, quaternion::Quaternion, transform::Transform, vector3::Vector3,
    vector4::Vector4,
};

#[repr(C)]
#[derive(Debug, PartialEq, Pod, Clone, Copy, Zeroable)]
pub struct Matrix4 {
    pub values: [[f32; 4]; 4],
}

impl Matrix4 {
    pub fn new_vectors(x: Vector3, y: Vector3, z: Vector3, p: Vector3) -> Self {
        Self {
            values: [
                [x.x, x.y, x.z, 0.0],
                [y.x, y.y, y.z, 0.0],
                [z.x, z.y, z.z, 0.0],
                [p.x, p.y, p.z, 1.0],
            ],
        }
    }

    pub fn zero() -> Self {
        Self {
            values: [[0.0; 4]; 4],
        }
    }

    pub fn identity() -> Self {
        Self {
            values: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }

    pub fn transform_vector(&self, v: Vector3) -> Vector4<f32> {
        Vector4::new(self * v, 0.0)
    }

    pub fn transform_point(&self, v: Vector3) -> Vector4<f32> {
        Vector4::new(self * v, 1.0)
    }

    pub fn transpose(&mut self) {
        self.values = self.transposed().values
    }

    pub fn transposed(&self) -> Self {
        let m = &self.values;
        Self {
            values: [
                [m[0][0], m[1][0], m[2][0], m[3][0]],
                [m[0][1], m[1][1], m[2][1], m[3][1]],
                [m[0][2], m[1][2], m[2][2], m[3][2]],
                [m[0][3], m[1][3], m[2][3], m[3][3]],
            ],
        }
    }

    pub fn det(&self) -> f32 {
        let m = &self.values;

        m[0][0] * m[1][1] * m[2][2] * m[3][3]
            - m[0][0] * m[1][1] * m[2][3] * m[3][2]
            - m[0][0] * m[1][2] * m[2][1] * m[3][3]
            + m[0][0] * m[1][2] * m[2][3] * m[3][1]
            + m[0][0] * m[1][3] * m[2][1] * m[3][2]
            - m[0][0] * m[1][3] * m[2][2] * m[3][1]
            - m[0][1] * m[1][0] * m[2][2] * m[3][3]
            + m[0][1] * m[1][0] * m[2][3] * m[3][2]
            + m[0][1] * m[1][2] * m[2][0] * m[3][3]
            - m[0][1] * m[1][2] * m[2][3] * m[3][0]
            - m[0][1] * m[1][3] * m[2][0] * m[3][2]
            + m[0][1] * m[1][3] * m[2][2] * m[3][0]
            + m[0][2] * m[1][0] * m[2][1] * m[3][3]
            - m[0][2] * m[1][0] * m[2][3] * m[3][1]
            - m[0][2] * m[1][1] * m[2][0] * m[3][3]
            + m[0][2] * m[1][1] * m[2][3] * m[3][0]
            + m[0][2] * m[1][3] * m[2][0] * m[3][1]
            - m[0][2] * m[1][3] * m[2][1] * m[3][0]
            - m[0][3] * m[1][0] * m[2][1] * m[3][2]
            + m[0][3] * m[1][0] * m[2][2] * m[3][1]
            + m[0][3] * m[1][1] * m[2][0] * m[3][2]
            - m[0][3] * m[1][1] * m[2][2] * m[3][0]
            - m[0][3] * m[1][2] * m[2][0] * m[3][1]
            + m[0][3] * m[1][2] * m[2][1] * m[3][0]
    }

    pub fn adjugate(&self) -> Matrix4 {
        let mut adj = [[0f32; 4]; 4];

        for i in 0..4 {
            for j in 0..4 {
                // Calculate the submatrix and its determinant
                let submatrix = self.submatrix(i, j);
                let det = submatrix.det();

                // Multiply the determinant by -1 if i + j is odd
                adj[j][i] = if (i + j) % 2 == 0 { det } else { -det };
            }
        }

        Matrix4 { values: adj }
    }

    pub fn inverse(&self) -> Option<Matrix4> {
        let det = self.det();
        if det == 0.0 {
            return None;
        }

        Some(&self.adjugate() * (1.0 / det))
    }

    pub fn invert(&mut self) {
        match self.inverse() {
            Some(i) => self.values = i.values,
            None => println!("Failed to invert matrix {:?}", self),
        }
    }

    pub fn perspective(fov: f32, aspect: f32, n: f32, f: f32) -> Self {
        let t = n * f32::tan(fov.to_radians());
        let b = -t;
        let r = t * aspect;
        let l = -r;
        Self {
            values: [
                [(2.0 * n) / (r - l), 0.0, 0.0, 0.0],
                [0.0, (2.0 * n) / (t - b), 0.0, 0.0],
                [
                    (r + l) / (r - l),
                    (t + b) / (t - b),
                    (-(f + n)) / (f - n),
                    -1.0,
                ],
                [0.0, 0.0, (-2.0 * f * n) / (f - n), 0.0],
            ],
        }
    }

    pub fn ortho(l: f32, r: f32, b: f32, t: f32, n: f32, f: f32) -> Self {
        assert_ne!(l, r);
        assert_ne!(b, t);
        assert_ne!(n, f);
        Self {
            values: [
                [2.0 / (r - l), 0.0, 0.0, 0.0],
                [0.0, 2.0 / (t - b), 0.0, 0.0],
                [0.0, 0.0, -2.0 / (f - n), 0.0],
                [
                    -((r + l) / (r - l)),
                    -((t + b) / (t - b)),
                    -((f + n) / (f - n)),
                    1.0,
                ],
            ],
        }
    }

    pub fn look_at(eye: Vector3, target: Vector3, up: Vector3) -> Option<Self> {
        let f = -(target - eye).normalized();
        let mut r = up.cross(f);
        if r.is_zero() {
            return None;
        }
        r.normalize();
        let u = f.cross(r).normalized();
        let t = Vector3 {
            x: -r.dot(eye),
            y: -u.dot(eye),
            z: -f.dot(eye),
        };
        Some(Self {
            values: [
                [r.x, u.x, f.x, 0.0],
                [r.y, u.y, f.y, 0.0],
                [r.z, u.z, f.z, 0.0],
                [t.x, t.y, t.z, 1.0],
            ],
        })
    }

    pub fn from_translation(v: Vector3) -> Self {
        Self {
            values: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [v.x, v.y, v.z, 1.0],
            ],
        }
    }

    pub fn right(&self) -> [f32; 3] {
        [self.values[0][0], self.values[0][1], self.values[0][2]]
    }

    pub fn up(&self) -> [f32; 3] {
        [self.values[1][0], self.values[1][1], self.values[1][2]]
    }

    pub fn forward(&self) -> [f32; 3] {
        [self.values[2][0], self.values[2][1], self.values[2][2]]
    }

    fn submatrix(&self, remove_row: usize, remove_col: usize) -> Matrix3 {
        let mut submatrix = [[0.0; 3]; 3];
        let mut row = 0;

        for i in 0..4 {
            if i == remove_row {
                continue;
            }

            let mut col = 0;

            for j in 0..4 {
                if j == remove_col {
                    continue;
                }

                submatrix[row][col] = self.values[i][j];
                col += 1;
            }

            row += 1;
        }

        Matrix3 { values: submatrix }
    }
}

impl<'a, 'b> Add<&'b Matrix4> for &'a Matrix4 {
    type Output = Matrix4;

    fn add(self, other: &'b Matrix4) -> Self::Output {
        let mut result = Self::Output::zero();
        for i in 0..4 {
            for j in 0..4 {
                result.values[i][j] = self.values[i][j] + other.values[i][j];
            }
        }
        result
    }
}

impl<'a, 'b> Sub<&'b Matrix4> for &'a Matrix4 {
    type Output = Matrix4;

    fn sub(self, other: &'b Matrix4) -> Self::Output {
        let mut result = Self::Output::zero();
        for i in 0..4 {
            for j in 0..4 {
                result.values[i][j] = self.values[i][j] - other.values[i][j];
            }
        }
        result
    }
}

impl<'a, 'b> Mul<&'b Matrix4> for &'a Matrix4 {
    type Output = Matrix4;

    fn mul(self, other: &'b Matrix4) -> Self::Output {
        let mut result = Self::Output::zero();
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    result.values[j][i] += self.values[k][i] * other.values[j][k];
                }
            }
        }
        result
    }
}

impl<'a> Mul<Vector3> for &'a Matrix4 {
    type Output = Vector3;

    fn mul(self, other: Vector3) -> Self::Output {
        let m = &self.values;
        Self::Output {
            x: m[0][0] * other.x + m[0][1] * other.x + m[0][2] * other.x,
            y: m[1][0] * other.y + m[1][1] * other.y + m[1][2] * other.y,
            z: m[2][0] * other.z + m[2][1] * other.z + m[2][2] * other.z,
        }
    }
}

impl<'a> Mul<f32> for &'a Matrix4 {
    type Output = Matrix4;

    fn mul(self, f: f32) -> Self::Output {
        let m = &self.values;
        Self::Output {
            values: [
                [m[0][0] * f, m[0][1] * f, m[0][2] * f, m[0][3] * f],
                [m[1][0] * f, m[1][1] * f, m[1][2] * f, m[1][3] * f],
                [m[2][0] * f, m[2][1] * f, m[2][2] * f, m[2][3] * f],
                [m[3][0] * f, m[3][1] * f, m[3][2] * f, m[3][3] * f],
            ],
        }
    }
}

impl<'a> Into<Quaternion> for &'a Matrix4 {
    fn into(self) -> Quaternion {
        let up: Vector3 = self.up().into();
        let up = up.normalized();
        let forward = self.forward().into();
        let right = up.cross(forward);
        let up = forward.cross(right);

        Quaternion::look_rotation(forward, up)
    }
}

impl From<Quaternion> for Matrix4 {
    fn from(value: Quaternion) -> Self {
        let right = value * Vector3::right();
        let up = value * Vector3::up();
        let forward = value
            * Vector3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            };
        Self {
            values: [
                [right.x, right.y, right.z, 0.0],
                [up.x, up.y, up.z, 0.0],
                [forward.x, forward.y, forward.z, 0.0],
                [0.0, 0.0, 0.0, 1.0],
            ],
        }
    }
}

impl<'a> Into<Transform> for &'a Matrix4 {
    fn into(self) -> Transform {
        let m = &self.values;

        let translation = Vector3 {
            x: m[3][0],
            y: m[3][1],
            z: m[3][2],
        };
        let rotation: Quaternion = self.into();

        let rotation_scale_matrix = Matrix4 {
            values: [
                [m[0][0], m[0][1], m[0][2], 0.0],
                [m[1][0], m[1][1], m[1][2], 0.0],
                [m[2][0], m[2][1], m[2][2], 0.0],
                [m[3][0], m[3][1], m[3][2], 1.0],
            ],
        };
        let inv_rotation_matrix: Matrix4 = rotation.inverse().into();
        let scale_skew_matrix = &rotation_scale_matrix * &inv_rotation_matrix;
        let scale = Vector3 {
            x: scale_skew_matrix.values[0][0],
            y: scale_skew_matrix.values[1][1],
            z: scale_skew_matrix.values[2][2],
        };

        Transform {
            translation,
            rotation,
            scale,
        }
    }
}

impl<'a> From<&'a Transform> for Matrix4 {
    fn from(value: &Transform) -> Self {
        let x = (value.rotation * Vector3::right()) * value.scale.x;
        let y = (value.rotation * Vector3::up()) * value.scale.y;
        let z = (value.rotation
            * Vector3 {
                x: 0.0,
                y: 0.0,
                z: 1.0,
            })
            * value.scale.z;
        let p = value.translation;

        Matrix4::new_vectors(x, y, z, p)
    }
}

impl From<[[f32; 4]; 4]> for Matrix4 {
    fn from(values: [[f32; 4]; 4]) -> Self {
        Matrix4 { values }
    }
}

impl Into<[[f32; 4]; 4]> for Matrix4 {
    fn into(self) -> [[f32; 4]; 4] {
        self.values
    }
}
