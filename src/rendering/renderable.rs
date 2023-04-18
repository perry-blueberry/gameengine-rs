use wgpu::{RenderPass, SurfaceError, VertexBufferLayout};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use super::{line::LineRender, model::TriangleModel, point::PointRender};

pub enum Renderable {
    Model(TriangleModel),
    Line(LineRender),
    Point(PointRender),
}

impl RenderableT for Renderable {
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        match self {
            Renderable::Model(m) => m.resize(new_size),
            Renderable::Line(l) => l.resize(new_size),
            Renderable::Point(p) => p.resize(new_size),
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match self {
            Renderable::Model(m) => m.input(event),
            Renderable::Line(l) => l.input(event),
            Renderable::Point(p) => p.input(event),
        }
    }
    fn update(&mut self) {
        match self {
            Renderable::Model(m) => m.update(),
            Renderable::Line(l) => l.update(),
            Renderable::Point(p) => p.update(),
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
        }
    }
}

pub trait RenderableT {
    fn resize(&mut self, new_size: PhysicalSize<u32>);
    fn input(&mut self, event: &WindowEvent) -> bool;
    fn update(&mut self);
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
