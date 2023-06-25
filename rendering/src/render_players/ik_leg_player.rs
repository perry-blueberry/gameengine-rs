use crate::{
    instance::Instance,
    {renderable::RenderableT, skeletal_model::SkeletalModel},
};
use animation::{clip::Clip, ik_leg::IkLeg, pose::Pose, skeleton::Skeleton, track::Vector3Track};
use collisions::triangle_ray::{Ray, Triangle};
use glam::{Quat, Vec3};
use math::glam_transform::{FromTo, LookRotation, Transform};

pub struct IkLegPlayer {
    walking_time: f32,
    model: Transform,
    motion_track: Vector3Track,
    triangles: Vec<Triangle>,
    sink_into_ground: f32,
    playback_time: f32,
    clip: Clip,
    skeleton: Skeleton,
    current_pose: Pose,
    left_leg: IkLeg,
    right_leg: IkLeg,
    last_model_y: f32,
    toe_length: f32,
    skeletal_model: SkeletalModel,
}

impl IkLegPlayer {
    pub fn new(
        motion_track: Vector3Track,
        triangles: Vec<Triangle>,
        sink_into_ground: f32,
        clip: Clip,
        skeleton: Skeleton,
        left_leg: IkLeg,
        right_leg: IkLeg,
        toe_length: f32,
        skeletal_model: SkeletalModel,
    ) -> Self {
        let current_pose = skeleton.rest_pose.clone();
        let ground_ray = Ray::new(Vec3::new(0.0, 11.0, 0.0));
        let mut model = Transform::default();
        for triangle in &triangles {
            if let Some(hit_point) = ground_ray.cast(triangle) {
                model.translation = hit_point;
                break;
            }
        }
        model.translation.y -= sink_into_ground;
        let last_model_y = model.translation.y;

        Self {
            walking_time: 0.0,
            model,
            motion_track,
            triangles,
            sink_into_ground,
            playback_time: 0.0,
            clip,
            skeleton,
            current_pose,
            left_leg,
            right_leg,
            last_model_y,
            toe_length,
            skeletal_model,
        }
    }
}

impl RenderableT for IkLegPlayer {
    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.skeletal_model.resize(new_size)
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.skeletal_model.input(event)
    }

    fn update(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        let ray_height = 2.1;
        self.walking_time += delta_time * 0.3;
        while self.walking_time > 6.0 {
            self.walking_time -= 6.0;
        }

        let last_y_position = self.model.translation.y;
        let mut current_position = self.motion_track.sample(self.walking_time, true);
        let mut next_position = self.motion_track.sample(self.walking_time + 0.1, true);
        current_position.y = last_y_position;
        next_position.y = last_y_position;
        self.model.translation = current_position;

        let direction = (next_position - current_position).normalize();
        let mut new_direction = Quat::look_rotation(direction, Vec3::Y);
        if self.model.rotation.dot(new_direction) < 0.0 {
            new_direction = new_direction * -1.0;
        }
        self.model.rotation = self.model.rotation.lerp(new_direction, delta_time * 10.0);
        let character_forward = self.model.rotation * Vec3::Z;

        let ground_ray = Ray::new(Vec3::new(
            self.model.translation.x,
            11.0,
            self.model.translation.z,
        ));
        for triangle in &self.triangles {
            if let Some(hit_point) = ground_ray.cast(triangle) {
                self.model.translation = hit_point - Vec3::new(0.0, self.sink_into_ground, 0.0);
                break;
            }
        }

        self.playback_time = self
            .clip
            .sample(&mut self.current_pose, self.playback_time + delta_time);
        let normalized_time =
            ((self.playback_time - self.clip.start_time) / self.clip.duration()).clamp(0.0, 1.0);
        let left_motion = self.left_leg.pin_track.sample(normalized_time, true);
        let right_motion = self.right_leg.pin_track.sample(normalized_time, true);

        let ankle_setup = |ankle_index: usize| {
            let world_ankle = self
                .model
                .combine(&self.current_pose.global_transform(ankle_index))
                .translation;
            let ankle_ray = Ray::new(world_ankle + Vec3::new(0.0, 2.0, 0.0));
            (world_ankle, ankle_ray, world_ankle)
        };

        let (mut world_left_ankle, left_ankle_ray, mut predictive_left_ankle) =
            ankle_setup(self.left_leg.ankle_index);

        let (mut world_right_ankle, right_ankle_ray, mut predictive_right_ankle) =
            ankle_setup(self.right_leg.ankle_index);

        let mut ground_reference = self.model.translation;
        let mut ankle_assignment =
            |triangle, ankle_ray: &Ray, world_ankle: &mut Vec3, predictive_ankle: &mut Vec3| {
                if let Some(hit_point) = ankle_ray.cast(triangle) {
                    if (hit_point - ankle_ray.origin).length_squared() < ray_height * ray_height {
                        *world_ankle = hit_point;
                    }
                    if hit_point.y < ground_reference.y {
                        ground_reference = hit_point - Vec3::new(0.0, self.sink_into_ground, 0.0);
                    }
                    *predictive_ankle = hit_point;
                }
            };
        for triangle in &self.triangles {
            ankle_assignment(
                triangle,
                &left_ankle_ray,
                &mut world_left_ankle,
                &mut predictive_left_ankle,
            );
            ankle_assignment(
                triangle,
                &right_ankle_ray,
                &mut world_right_ankle,
                &mut predictive_right_ankle,
            );
        }

        self.model.translation.y = self.last_model_y;
        self.model.translation = self
            .model
            .translation
            .lerp(ground_reference, delta_time * 10.0);
        self.last_model_y = self.model.translation.y;

        let mut solve_legs = |world_ankle: &mut Vec3, predictive_ankle, motion, leg: &mut IkLeg| {
            *world_ankle = world_ankle.lerp(predictive_ankle, motion);

            leg.solve(&self.model, &self.current_pose, *world_ankle);
            self.current_pose.blend(
                &self.current_pose.clone(),
                &leg.ik_pose,
                1.0,
                Some(leg.hip_index),
            );

            let ankle_world = self
                .model
                .combine(&self.current_pose.global_transform(leg.ankle_index));

            let world_toe = self
                .model
                .combine(&self.current_pose.global_transform(leg.toe_index))
                .translation;
            let toe_target = world_toe;
            let predictive_toe = world_toe;

            let mut origin = ankle_world.translation;
            origin.y = world_toe.y;
            let toe_ray = Ray::new(origin + character_forward * self.toe_length + Vec3::Y);
            (toe_ray, toe_target, predictive_toe, world_toe, ankle_world)
        };

        let (
            left_toe_ray,
            mut left_toe_target,
            mut predictive_left_toe,
            world_left_toe,
            left_ankle_world,
        ) = solve_legs(
            &mut world_left_ankle,
            predictive_left_ankle,
            left_motion,
            &mut self.left_leg,
        );
        let (
            right_toe_ray,
            mut right_toe_target,
            mut predictive_right_toe,
            world_right_toe,
            right_ankle_world,
        ) = solve_legs(
            &mut world_right_ankle,
            predictive_right_ankle,
            right_motion,
            &mut self.right_leg,
        );

        let ankle_ray_height = 1.1;
        let toe_assignment =
            |triangle, toe_ray: &Ray, toe_target: &mut Vec3, predictive_toe: &mut Vec3| {
                if let Some(hit_point) = toe_ray.cast(triangle) {
                    if (hit_point - toe_ray.origin).length_squared()
                        < ankle_ray_height * ankle_ray_height
                    {
                        *toe_target = hit_point;
                    }
                    *predictive_toe = hit_point;
                }
            };
        for triangle in &self.triangles {
            toe_assignment(
                triangle,
                &left_toe_ray,
                &mut left_toe_target,
                &mut predictive_left_toe,
            );
            toe_assignment(
                triangle,
                &right_toe_ray,
                &mut right_toe_target,
                &mut predictive_right_toe,
            );
        }

        let mut fix_ankles = |toe_target: &mut Vec3,
                              predictive_toe,
                              motion,
                              world_toe: Vec3,
                              ankle_world: &Transform,
                              ankle_index| {
            *toe_target = toe_target.lerp(predictive_toe, motion);

            let ankle_to_current_toe: Vec3 = world_toe - ankle_world.translation;
            let ankle_to_desired_toe = *toe_target - ankle_world.translation;
            if ankle_to_current_toe.dot(ankle_to_desired_toe) > 0.00001 {
                let ankle_rotator = Quat::from_to(ankle_to_current_toe, ankle_to_desired_toe);
                let mut ankle_local = self.current_pose.local_transform(ankle_index).clone();
                /* HERE? */
                let world_rotated_ankle = ankle_rotator * ankle_world.rotation;
                let local_rotated_ankle = ankle_world.rotation.inverse() * world_rotated_ankle;

                ankle_local.rotation = ankle_local.rotation * local_rotated_ankle;
                self.current_pose
                    .set_local_transform(ankle_index, ankle_local);
            }
        };

        fix_ankles(
            &mut left_toe_target,
            predictive_left_toe,
            left_motion,
            world_left_toe,
            &left_ankle_world,
            self.left_leg.ankle_index,
        );
        fix_ankles(
            &mut right_toe_target,
            predictive_right_toe,
            right_motion,
            world_right_toe,
            &right_ankle_world,
            self.right_leg.ankle_index,
        );

        let mut pose_palette = self.current_pose.matrix_palette();
        {
            for (i, p) in pose_palette.iter_mut().enumerate() {
                *p = *p * self.skeleton.inverse_bind_pose()[i];
            }
        }
        queue.write_buffer(
            &self.skeletal_model.animated_buffer,
            0,
            bytemuck::cast_slice(&pose_palette),
        );
        let instance_data = vec![Instance {
            position: self.model.translation.into(),
            rotation: self.model.rotation.into(),
        }]
        .iter()
        .map(Instance::to_raw)
        .collect::<Vec<_>>();
        queue.write_buffer(
            &self.skeletal_model.instance_buffer,
            0,
            bytemuck::cast_slice(&instance_data),
        )
    }

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        self.skeletal_model.render(render_pass)
    }
}
