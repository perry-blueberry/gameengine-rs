use crate::{
    instance::Instance,
    texture,
    {
        model::{DrawModel, Model},
        renderable::RenderableT,
        skeletal_model::{new_skeletal_pipeline, SkeletalModelBase, SkeletalVertex},
    },
};
use animation::{clip::Clip, pose::Pose, skeleton::Skeleton};
use std::sync::{Arc, RwLock};

use anyhow::{Ok, Result};
use gltf::Material;
use wgpu::{BindGroup, Device, RenderPipeline, SurfaceConfiguration};

struct Base {
    render_pipeline: RenderPipeline,
    model: Model<SkeletalVertex>,
    camera_bind_group: BindGroup,
    pose_bind_group: BindGroup,
    instance_buffer: wgpu::Buffer,
    animated_buffer: wgpu::Buffer,
    skeleton: Arc<Skeleton>,
}

struct BlendBetweenClips {
    pose: Pose,
    clip_a: Clip,
    clip_b: Clip,
    time_a: f32,
    time_b: f32,
    pose_a: Pose,
    pose_b: Pose,
    blend_time: f32,
    invert_blend: bool,
}

struct LayeredAnimation {
    current_pose: Pose,
    add_pose: Pose,
    additive_base: Pose,
    clips: Vec<Clip>,
    clip_index: usize,
    additive_index: usize,
    playback_time: f32,
    additive_time: f32,
    additive_direction: f32,
}

enum Method {
    BlendBetweenClips(BlendBetweenClips),
    LayeredAnimation(LayeredAnimation),
}

pub struct BlenderPlayer {
    base: Base,
    method: Method,
}

impl BlenderPlayer {
    pub async fn new_blend_between_clips<'a>(
        vertices: Vec<SkeletalVertex>,
        original_positions: Vec<[f32; 3]>,
        original_normals: Vec<[f32; 3]>,
        indices: Vec<u32>,
        model_name: &str,
        device: &Device,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
        material: Material<'a>,
        diffuse_texture: Arc<RwLock<texture::Texture>>,
        skeleton: Arc<Skeleton>,
        instances: Arc<RwLock<Vec<Instance>>>,
        pose: Pose,
        clip_a: Clip,
        clip_b: Clip,
        time_a: f32,
        time_b: f32,
        pose_a: Pose,
        pose_b: Pose,
    ) -> Result<Self> {
        let SkeletalModelBase {
            render_pipeline,
            model,
            camera_bind_group,
            pose_bind_group,
            original_positions: _,
            original_normals: _,
            instance_buffer,
            animated_buffer,
        } = {
            let instances = instances.read().unwrap();
            new_skeletal_pipeline(
                vertices,
                original_positions,
                original_normals,
                indices,
                model_name,
                device,
                config,
                camera_buffer,
                &material,
                diffuse_texture,
                &instances,
            )
        };
        Ok(Self {
            base: Base {
                render_pipeline,
                model,
                camera_bind_group,
                pose_bind_group,
                instance_buffer,
                animated_buffer,
                skeleton,
            },
            method: Method::BlendBetweenClips(BlendBetweenClips {
                pose,
                clip_a,
                clip_b,
                time_a,
                time_b,
                pose_a,
                pose_b,
                blend_time: 0.0,
                invert_blend: false,
            }),
        })
    }

    pub async fn new_layered_animation<'a>(
        vertices: Vec<SkeletalVertex>,
        original_positions: Vec<[f32; 3]>,
        original_normals: Vec<[f32; 3]>,
        indices: Vec<u32>,
        model_name: &str,
        device: &Device,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
        material: Material<'a>,
        diffuse_texture: Arc<RwLock<texture::Texture>>,
        skeleton: Arc<Skeleton>,
        instances: Arc<RwLock<Vec<Instance>>>,
        current_pose: Pose,
        add_pose: Pose,
        additive_base: Pose,
        clips: Vec<Clip>,
        clip_index: usize,
        additive_index: usize,
    ) -> Result<Self> {
        let SkeletalModelBase {
            render_pipeline,
            model,
            camera_bind_group,
            pose_bind_group,
            original_positions: _,
            original_normals: _,
            instance_buffer,
            animated_buffer,
        } = {
            let instances = instances.read().unwrap();
            new_skeletal_pipeline(
                vertices,
                original_positions,
                original_normals,
                indices,
                model_name,
                device,
                config,
                camera_buffer,
                &material,
                diffuse_texture,
                &instances,
            )
        };
        Ok(Self {
            base: Base {
                render_pipeline,
                model,
                camera_bind_group,
                pose_bind_group,
                instance_buffer,
                animated_buffer,
                skeleton,
            },
            method: Method::LayeredAnimation(LayeredAnimation {
                current_pose,
                add_pose,
                additive_base,
                clips,
                clip_index,
                additive_index,
                playback_time: 0.0,
                additive_time: 0.0,
                additive_direction: 1.0,
            }),
        })
    }
}

impl RenderableT for BlenderPlayer {
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        /* let delta_time = 0.2; */
        let mut pose_palette = match &mut self.method {
            Method::BlendBetweenClips(BlendBetweenClips {
                pose,
                clip_a,
                clip_b,
                time_a,
                time_b,
                pose_a,
                pose_b,
                blend_time,
                invert_blend,
            }) => {
                *time_a = clip_a.sample(pose_a, *time_a + delta_time);
                *time_b = clip_b.sample(pose_b, *time_b + delta_time);

                let mut bt = blend_time.clamp(0.0, 1.0);
                if *invert_blend {
                    bt = 1.0 - bt;
                }
                pose.blend(&pose_a, &pose_b, bt, None);
                *blend_time += delta_time;
                if *blend_time > 2.0 {
                    *blend_time = 0.0;
                    *invert_blend = !*invert_blend;
                    *pose = self.base.skeleton.rest_pose.clone();
                }
                pose.matrix_palette()
            }
            Method::LayeredAnimation(LayeredAnimation {
                current_pose,
                add_pose,
                additive_base,
                clips,
                clip_index,
                additive_index,
                playback_time,
                additive_time,
                additive_direction,
            }) => {
                *additive_time += delta_time * *additive_direction;
                *additive_time = additive_time.clamp(0.0, 1.0);
                if *additive_time == 0.0 || *additive_time == 1.0 {
                    *additive_direction *= -1.0;
                }
                *playback_time =
                    clips[*clip_index].sample(current_pose, *playback_time + delta_time);
                /* *current_pose = self.base.skeleton.rest_pose.clone(); */
                let additive_clip = &mut clips[*additive_index];
                additive_clip.looping = false;
                let time = additive_clip.start_time + (additive_clip.duration() * *additive_time);
                additive_clip.sample(add_pose, time);
                current_pose.add(&current_pose.clone(), add_pose, additive_base, None);
                current_pose.matrix_palette()
            }
        };
        for (i, p) in pose_palette.iter_mut().enumerate() {
            *p = *p * self.base.skeleton.inverse_bind_pose()[i];
        }
        queue.write_buffer(
            &self.base.animated_buffer,
            0,
            bytemuck::cast_slice(&pose_palette),
        );
    }

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        render_pass.set_vertex_buffer(1, self.base.instance_buffer.slice(..));
        render_pass.set_pipeline(&self.base.render_pipeline);
        render_pass.draw_model_instanced(
            &self.base.model,
            0..1,
            vec![
                (1, &self.base.camera_bind_group),
                (2, &self.base.pose_bind_group),
            ],
        );
        std::result::Result::Ok(())
    }
}
