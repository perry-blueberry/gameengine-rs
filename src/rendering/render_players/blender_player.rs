use crate::{
    animation::{clip::Clip, pose::Pose, skeleton::Skeleton},
    rendering::{
        model::{DrawModel, Model},
        renderable::RenderableT,
        skeletal_model::{new_skeletal_pipeline, SkeletalVertex},
    },
    texture,
};

use anyhow::{Ok, Result};
use gltf::Material;
use wgpu::{BindGroup, Device, RenderPipeline, SurfaceConfiguration};

pub struct BlenderPlayer {
    skeleton: Skeleton,
    pose: Pose,
    clip_a: Clip,
    clip_b: Clip,
    time_a: f32,
    time_b: f32,
    pose_a: Pose,
    pose_b: Pose,
    blend_time: f32,
    invert_blend: bool,
    render_pipeline: RenderPipeline,
    model: Model<SkeletalVertex>,
    camera_bind_group: BindGroup,
    pose_bind_group: BindGroup,
    instance_buffer: wgpu::Buffer,
    animated_buffer: wgpu::Buffer,
}

impl BlenderPlayer {
    pub async fn new<'a>(
        vertices: Vec<SkeletalVertex>,
        original_positions: Vec<[f32; 3]>,
        original_normals: Vec<[f32; 3]>,
        indices: Vec<u32>,
        model_name: &str,
        device: &Device,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
        material: Material<'a>,
        diffuse_texture: texture::Texture,
        skeleton: Skeleton,
        pose: Pose,
        clip_a: Clip,
        clip_b: Clip,
        time_a: f32,
        time_b: f32,
        pose_a: Pose,
        pose_b: Pose,
    ) -> Result<Self> {
        let (
            render_pipeline,
            model,
            camera_bind_group,
            pose_bind_group,
            _original_positions,
            _original_normals,
            instance_buffer,
            animated_buffer,
        ) = new_skeletal_pipeline(
            vertices,
            original_positions,
            original_normals,
            indices,
            model_name,
            device,
            config,
            camera_buffer,
            material,
            diffuse_texture,
        )
        .await;
        Ok(Self {
            skeleton,
            pose,
            clip_a,
            clip_b,
            time_a,
            time_b,
            pose_a,
            pose_b,
            blend_time: 0.0,
            invert_blend: false,
            render_pipeline,
            model,
            camera_bind_group,
            pose_bind_group,
            instance_buffer,
            animated_buffer,
        })
    }
}

impl RenderableT for BlenderPlayer {
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        self.time_a = self
            .clip_a
            .sample(&mut self.pose_a, self.time_a + delta_time);
        self.time_b = self
            .clip_b
            .sample(&mut self.pose_b, self.time_b + delta_time);

        let mut bt = self.blend_time.clamp(0.0, 1.0);
        if self.invert_blend {
            bt = 1.0 - bt;
        }
        self.pose.blend(&self.pose_a, &self.pose_b, bt, None);
        self.blend_time += delta_time;
        if self.blend_time > 2.0 {
            self.blend_time = 0.0;
            self.invert_blend = !self.invert_blend;
            self.pose = self.skeleton.rest_pose.clone();
        }
        let mut pose_palette = self.pose.matrix_palette();
        for (i, p) in pose_palette.iter_mut().enumerate() {
            *p = *p * self.skeleton.inverse_bind_pose()[i];
        }
        queue.write_buffer(
            &self.animated_buffer,
            0,
            bytemuck::cast_slice(&pose_palette),
        );
    }

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_model_instanced(
            &self.model,
            0..1,
            vec![(1, &self.camera_bind_group), (2, &self.pose_bind_group)],
        );
        std::result::Result::Ok(())
    }
}
