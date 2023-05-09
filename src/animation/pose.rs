use cgmath::{Decomposed, Matrix4, Quaternion, Transform as Tf, Vector3};

pub(crate) type Transform = Decomposed<Vector3<f32>, Quaternion<f32>>;

#[derive(Debug, PartialEq, Clone)]
pub struct Pose {
    joints: Vec<Transform>,
    parents: Vec<Option<usize>>,
}

impl Pose {
    pub fn new() -> Self {
        Self {
            joints: vec![],
            parents: vec![],
        }
    }
    pub fn len(&self) -> usize {
        self.joints.len()
    }

    pub fn local_transform(&self, idx: usize) -> Transform {
        self.joints[idx]
    }

    pub fn set_local_transform(&mut self, idx: usize, tf: Transform) {
        self.joints[idx] = tf;
    }

    pub fn add_local_transform(&mut self, tf: Transform) {
        self.joints.push(tf);
    }

    pub fn global_transform(&self, idx: usize) -> Transform {
        let mut res = self.joints[idx];
        if let Some(mut parent_index) = self.parents[idx] {
            res = self.joints[parent_index].concat(&res);
            while let Some(parent) = self.parents[parent_index] {
                parent_index = parent;
                res = self.joints[parent_index].concat(&res);
            }
        }
        res
    }

    pub fn matrix_palette(&self) -> Vec<Matrix4<f32>> {
        let mut result = Vec::with_capacity(self.len());
        for i in 0..self.len() {
            result.push(self.global_transform(i).into());
        }
        result
    }

    pub fn parent(&self, idx: usize) -> Option<usize> {
        self.parents[idx]
    }

    pub fn set_parent(&mut self, idx: usize, parent: usize) {
        self.parents[idx] = Some(parent);
    }

    pub fn unset_parent(&mut self, idx: usize) {
        self.parents[idx] = None
    }

    pub fn add_parent(&mut self, parent: Option<usize>) {
        self.parents.push(parent);
    }
}
