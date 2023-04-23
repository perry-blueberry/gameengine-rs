use cgmath::{Decomposed, Quaternion, Transform as Tf, Vector3};

type Transform = Decomposed<Vector3<f32>, Quaternion<f32>>;

#[derive(Debug, PartialEq)]
pub struct Pose {
    joints: Vec<Transform>,
    parents: Vec<Option<usize>>,
}

impl Pose {
    pub fn len(&self) -> usize {
        self.joints.len()
    }

    pub fn local_transform(&self, idx: usize) -> Transform {
        self.joints[idx]
    }

    pub fn set_local_transform(&mut self, idx: usize, tf: Transform) {
        self.joints[idx] = tf;
    }

    pub fn global_transform(&self, idx: usize) -> Transform {
        let mut res = self.joints[idx];
        let mut parent_index =
            self.parents[idx].expect(&format!("idx {} does not have a parent", idx));
        while let Some(parent) = self.parents[parent_index] {
            parent_index = parent;
            res = self.joints[parent].concat(&res);
        }
        res
    }

    pub fn matrix_palette(&self) {}

    pub fn parent(&self, idx: usize) -> Option<usize> {
        self.parents[idx]
    }

    pub fn set_parent(&mut self, idx: usize, parent: usize) {
        self.parents[idx] = Some(parent);
    }

    pub fn unset_parent(&mut self, idx: usize) {
        self.parents[idx] = None
    }
}
