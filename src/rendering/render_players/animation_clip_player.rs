use wgpu::{
    CompareFunction, DepthBiasState, DepthStencilState, Device, Queue, StencilState,
    SurfaceConfiguration,
};

use crate::{
    animation::{clip::Clip, pose::Pose},
    texture,
};

use super::super::{
    line::LineRender,
    renderable::{RenderableT, SimpleVertex},
};
pub struct AnimationClipPlayer {
    line_render: LineRender,
    playback_time: f32,
    clip: Clip,
    pose: Pose,
}

impl AnimationClipPlayer {
    pub fn new(
        clip: Clip,
        device: &Device,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
        pose: Pose,
    ) -> Self {
        let vertices = from_pose(&pose, [0.0, 1.0, 0.0]);
        let line_render = LineRender::new(
            vertices,
            device,
            config,
            camera_buffer,
            Some(DepthStencilState {
                format: texture::Texture::DEPTH_FORMAT,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: StencilState::default(),
                bias: DepthBiasState::default(),
            }),
        );

        Self {
            line_render,
            playback_time: 0.0,
            clip,
            pose,
        }
    }
}

impl RenderableT for AnimationClipPlayer {
    fn update(&mut self, delta_time: f32, queue: &Queue) {
        let time = self.playback_time + delta_time;
        self.playback_time = self.clip.sample(&mut self.pose, time);
        let vertices = from_pose(&self.pose, [0.0, 1.0, 0.0]);
        self.line_render.update_lines(&vertices, queue);
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        self.line_render.resize(new_size)
    }

    fn input(&mut self, event: &winit::event::WindowEvent) -> bool {
        self.line_render.input(event)
    }

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        self.line_render.render(render_pass)
    }
}

pub fn from_pose(pose: &Pose, color: [f32; 3]) -> Vec<SimpleVertex> {
    let mut points = vec![];
    for i in 0..pose.len() {
        if let Some(parent) = pose.parent(i) {
            points.push(SimpleVertex {
                position: pose.global_transform(i).translation.into(),
                color,
            });
            points.push(SimpleVertex {
                position: pose.global_transform(parent).translation.into(),
                color,
            });
        }
    }
    points
}
