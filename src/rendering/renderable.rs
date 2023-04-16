use wgpu::{RenderPass, SurfaceError};
use winit::{dpi::PhysicalSize, event::WindowEvent};

use super::{line::LineRender, model::TriangleModel};

pub enum Renderable {
    Model(TriangleModel),
    Line(LineRender),
}

impl RenderableT for Renderable {
    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        match self {
            Renderable::Model(m) => m.resize(new_size),
            Renderable::Line(l) => l.resize(new_size),
        }
    }
    fn input(&mut self, event: &WindowEvent) -> bool {
        match self {
            Renderable::Model(m) => m.input(event),
            Renderable::Line(l) => l.input(event),
        }
    }
    fn update(&mut self) {
        match self {
            Renderable::Model(m) => m.update(),
            Renderable::Line(l) => l.update(),
        }
    }
    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut RenderPass<'b>,
    ) -> Result<(), SurfaceError> {
        match self {
            Renderable::Model(m) => m.render(render_pass),
            Renderable::Line(l) => l.render(render_pass),
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
