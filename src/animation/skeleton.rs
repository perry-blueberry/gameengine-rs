use glam::Mat4;

use super::pose::Pose;

#[derive(Clone)]
pub struct Skeleton {
    pub rest_pose: Pose,
    pub bind_pose: Pose,
    joint_names: Vec<String>,
    pub inverse_bind_pose: Vec<Mat4>,
}

impl Skeleton {
    pub fn new(rest_pose: Pose, bind_pose: Pose, joint_names: Vec<String>) -> Self {
        let mut this = Self {
            rest_pose,
            bind_pose,
            joint_names,
            inverse_bind_pose: vec![],
        };
        this.update_inverse_bind_pose();
        this
    }

    pub fn joint_name(&self, idx: usize) -> &str {
        &self.joint_names[idx]
    }

    pub fn inverse_bind_pose(&self) -> &Vec<Mat4> {
        &self.inverse_bind_pose
    }

    fn update_inverse_bind_pose(&mut self) {
        self.inverse_bind_pose = vec![];
        self.inverse_bind_pose.reserve(self.bind_pose.len());
        for i in 0..self.bind_pose.len() {
            let world = self.bind_pose.global_transform(i);
            self.inverse_bind_pose.push(
                world
                    .inverse()
                    /* .expect(&format!("Failed to inverse world {:?}", world)) */
                    .into(),
            );
        }
    }
}
