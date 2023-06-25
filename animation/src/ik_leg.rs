use crate::{fabrik_solver::FabrikSolver, pose::Pose, skeleton::Skeleton, track::ScalarTrack};
use glam::{Quat, Vec3};
use math::glam_transform::Transform;

pub struct IkLeg {
    pub pin_track: ScalarTrack,
    solver: FabrikSolver,
    pub ik_pose: Pose,
    pub hip_index: usize,
    pub knee_index: usize,
    pub ankle_index: usize,
    pub toe_index: usize,
    ankle_to_ground_offset: f32,
}

impl IkLeg {
    pub fn new(
        hip: &str,
        knee: &str,
        ankle: &str,
        toe: &str,
        ankle_to_ground_offset: f32,
        skeleton: &Skeleton,
    ) -> Self {
        let solver = {
            let mut solver = FabrikSolver::new();
            solver.resize(3);
            solver
        };
        let mut hip_index: Option<usize> = None;
        let mut knee_index: Option<usize> = None;
        let mut ankle_index: Option<usize> = None;
        let mut toe_index: Option<usize> = None;
        for i in 0..skeleton.rest_pose.len() {
            let joint_name = skeleton.joint_name(i);
            if joint_name == hip {
                hip_index = Some(i);
            } else if joint_name == knee {
                knee_index = Some(i);
            } else if joint_name == ankle {
                ankle_index = Some(i);
            } else if joint_name == toe {
                toe_index = Some(i);
            }
        }
        Self {
            pin_track: ScalarTrack::new(),
            solver,
            ik_pose: Pose::new(),
            hip_index: hip_index.unwrap(),
            knee_index: knee_index.unwrap(),
            ankle_index: ankle_index.unwrap(),
            toe_index: toe_index.unwrap(),
            ankle_to_ground_offset,
        }
    }

    pub fn solve(&mut self, model: &Transform, pose: &Pose, ankle_target_position: Vec3) {
        self.solver
            .set_local_transform(0, model.combine(&pose.global_transform(self.hip_index)));
        self.solver
            .set_local_transform(1, pose.local_transform(self.knee_index).clone());
        self.solver
            .set_local_transform(2, pose.local_transform(self.ankle_index).clone());
        self.ik_pose = pose.clone();

        let target = Transform::new(
            ankle_target_position + Vec3::Y * self.ankle_to_ground_offset,
            Quat::default(),
            Vec3::ONE,
        );
        self.solver.solve(&target);

        let root_world =
            model.combine(&pose.global_transform(pose.parent(self.hip_index).unwrap()));
        self.ik_pose.set_local_transform(
            self.hip_index,
            root_world.inverse().combine(self.solver.local_transform(0)),
        );
        self.ik_pose
            .set_local_transform(self.knee_index, self.solver.local_transform(1).clone());
        self.ik_pose
            .set_local_transform(self.ankle_index, self.solver.local_transform(2).clone());
    }
}
