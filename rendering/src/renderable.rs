use wgpu::{Queue, RenderPass, SurfaceError, VertexBufferLayout};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use super::{
    line::LineRender,
    model::TriangleModel,
    point::PointRender,
    render_players::{
        animation_clip_player::AnimationClipPlayer, blender_player::BlenderPlayer,
        ik_leg_player::IkLegPlayer, ik_player::IkPlayer,
    },
    skeletal_model::SkeletalModel,
};

pub enum Renderable {
    Model(TriangleModel),
    Line(LineRender),
    Point(PointRender),
    AnimationClipPlayer(AnimationClipPlayer),
    SkeletalModel(SkeletalModel),
    BlenderPlayer(BlenderPlayer),
    IkPlayer(IkPlayer),
    IkLegPlayer(IkLegPlayer),
}

//TODO: Use a crate (proxy_enum, enum_dispatch) or create macro
impl RenderableT for Renderable {
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        match self {
            Renderable::Model(m) => m.resize(new_size),
            Renderable::Line(l) => l.resize(new_size),
            Renderable::Point(p) => p.resize(new_size),
            Renderable::AnimationClipPlayer(a) => a.resize(new_size),
            Renderable::SkeletalModel(m) => m.resize(new_size),
            Renderable::BlenderPlayer(p) => p.resize(new_size),
            Renderable::IkPlayer(p) => p.resize(new_size),
            Renderable::IkLegPlayer(p) => p.resize(new_size),
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match self {
            Renderable::Model(m) => m.input(event),
            Renderable::Line(l) => l.input(event),
            Renderable::Point(p) => p.input(event),
            Renderable::AnimationClipPlayer(a) => a.input(event),
            Renderable::SkeletalModel(m) => m.input(event),
            Renderable::BlenderPlayer(p) => p.input(event),
            Renderable::IkPlayer(p) => p.input(event),
            Renderable::IkLegPlayer(p) => p.input(event),
        }
    }
    fn update(&mut self, delta_time: f32, queue: &Queue) {
        match self {
            Renderable::Model(m) => m.update(delta_time, queue),
            Renderable::Line(l) => l.update(delta_time, queue),
            Renderable::Point(p) => p.update(delta_time, queue),
            Renderable::AnimationClipPlayer(a) => a.update(delta_time, queue),
            Renderable::SkeletalModel(m) => m.update(delta_time, queue),
            Renderable::BlenderPlayer(p) => p.update(delta_time, queue),
            Renderable::IkPlayer(p) => p.update(delta_time, queue),
            Renderable::IkLegPlayer(p) => p.update(delta_time, queue),
        }
    }
    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut RenderPass<'b>,
    ) -> Result<(), SurfaceError> {
        match self {
            Renderable::Model(m) => m.render(render_pass),
            Renderable::Line(l) => l.render(render_pass),
            Renderable::Point(p) => p.render(render_pass),
            Renderable::AnimationClipPlayer(a) => a.render(render_pass),
            Renderable::SkeletalModel(m) => m.render(render_pass),
            Renderable::BlenderPlayer(p) => p.render(render_pass),
            Renderable::IkPlayer(p) => p.render(render_pass),
            Renderable::IkLegPlayer(p) => p.render(render_pass),
        }
    }
}

pub trait RenderableT {
    fn resize(&mut self, new_size: PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self, delta_time: f32, queue: &Queue);
    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut RenderPass<'b>,
    ) -> Result<(), SurfaceError>;
}

pub trait Vertex {
    fn desc<'a>() -> VertexBufferLayout<'a>;
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct SimpleVertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex for SimpleVertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<SimpleVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub trait Updatable {
    fn update(&mut self, delta_time: f32);
}
