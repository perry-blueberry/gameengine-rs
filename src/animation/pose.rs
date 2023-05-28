use glam::Mat4;

use crate::math::glam_transform::Transform;

use super::{clip::Clip, skeleton::Skeleton};

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

    pub fn make_additive(skeleton: &Skeleton, clip: &Clip) -> Self {
        let mut result = skeleton.rest_pose.clone();
        clip.sample(&mut result, clip.start_time);
        result
    }

    pub fn len(&self) -> usize {
        self.joints.len()
    }

    pub fn local_transform(&self, idx: usize) -> &Transform {
        &self.joints[idx]
    }

    pub fn set_local_transform(&mut self, idx: usize, tf: Transform) {
        self.joints[idx] = tf;
    }

    pub fn add_local_transform(&mut self, tf: Transform) {
        self.joints.push(tf);
    }

    pub fn global_transform(&self, idx: usize) -> Transform {
        let mut res = self.joints[idx].clone();
        if let Some(mut parent_index) = self.parents[idx] {
            res = self.joints[parent_index].combine(&res);
            while let Some(parent) = self.parents[parent_index] {
                parent_index = parent;
                res = self.joints[parent_index].combine(&res);
            }
        }
        res
    }

    pub fn matrix_palette(&self) -> Vec<Mat4> {
        let mut result = vec![Mat4::IDENTITY; self.len()];
        let mut i = 0;

        // A "cache" if parents are stored before children
        for j in i..self.len() {
            let parent_index = self.parent(j);
            if parent_index.is_some() && parent_index.unwrap() > j {
                break;
            }
            let mut global: Mat4 = self.joints[j].clone().into();
            if let Some(parent_index) = parent_index {
                global = result[parent_index] * global;
            }
            result[j] = global;
            i = j;
        }
        // Fallback to calculating the global transform for the rest
        for j in i..self.len() {
            result[j] = self.global_transform(j).into();
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

    pub fn blend(&mut self, a: &Self, b: &Self, t: f32, blend_root: Option<usize>) {
        for i in 0..self.len() {
            // Only check hierarchy if a blend_root is present
            if let Some(blend_root) = blend_root {
                if !self.is_in_hierarchy(blend_root, i) {
                    continue;
                }
            }
            self.set_local_transform(i, a.local_transform(i).mix(b.local_transform(i), t));
        }
    }

    pub fn add(
        &mut self,
        in_pose: &Pose,
        add_pose: &Pose,
        base_pose: &Pose,
        blend_root: Option<usize>,
    ) {
        for i in 0..add_pose.len() {
            if let Some(blend_root) = blend_root {
                if !add_pose.is_in_hierarchy(blend_root, i) {
                    continue;
                }
            }
            let input = in_pose.local_transform(i);
            let additive = add_pose.local_transform(i);
            let additive_base = base_pose.local_transform(i);
            let result = Transform {
                translation: input.translation + (additive.translation - additive_base.translation),
                rotation: (input.rotation * (additive_base.rotation.inverse() * additive.rotation)),
                scale: input.scale + (additive.scale - additive_base.scale),
            };
            self.set_local_transform(i, result);
        }
    }

    fn is_in_hierarchy(&self, parent: usize, search: usize) -> bool {
        if search == parent {
            return true;
        }
        let mut current_parent = search;
        while let Some(new_parent) = self.parent(current_parent) {
            if new_parent == parent {
                return true;
            }
            current_parent = new_parent;
        }
        false
    }
}
