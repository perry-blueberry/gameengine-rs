pub struct Matrix3 {
    pub values: [[f32; 3]; 3],
}

impl Matrix3 {
    // Helper function to calculate the determinant of a 3x3 matrix
    pub fn det(&self) -> f32 {
        let m = &self.values;
        m[0][0] * (m[1][1]*m[2][2] - m[1][2]*m[2][1]) -
        m[0][1] * (m[1][0]*m[2][2] - m[1][2]*m[2][0]) +
        m[0][2] * (m[1][0]*m[2][1] - m[1][1]*m[2][0])
    }
}