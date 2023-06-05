use crate::{
    animation::{
        fabrik_solver::FabrikSolver, track::Vector3Track, transform_track::TransformTrack,
    },
    math::glam_transform::Transform,
    rendering::{
        line::LineRender,
        point::PointRender,
        renderable::{RenderableT, SimpleVertex},
    },
};

pub struct IkPlayer {
    solver: FabrikSolver,
    target_path: TransformTrack,
    solver_point_renderer: PointRender,
    solver_line_renderer: LineRender,
    target: Transform,
    play_time: f32,
}

impl IkPlayer {
    pub fn new(
        solver: FabrikSolver,
        target_path: TransformTrack,
        solver_point_renderer: PointRender,
        solver_line_renderer: LineRender,
    ) -> Self {
        Self {
            solver,
            target_path,
            target: Transform::default(),
            play_time: 0.0,
            solver_point_renderer,
            solver_line_renderer,
        }
    }
}

impl RenderableT for IkPlayer {
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        self.play_time += delta_time;
        if self.play_time > self.target_path.end_time().unwrap() {
            self.play_time -= self.target_path.end_time().unwrap();
        }
        self.target = self
            .target_path
            .sample(self.target.clone(), self.play_time, true);
        self.solver.solve(&self.target);
        let mut points = Vec::with_capacity(self.solver.len());
        for i in 0..self.solver.len() {
            points.push(SimpleVertex {
                position: self.solver.global_transform(i).translation.into(),
                color: [1.0, 0.0, 1.0],
            });
        }
        let mut lines = vec![points[0]];
        for i in 1..points.len() - 1 {
            lines.push(points[i]);
            lines.push(points[i]);
        }
        lines.push(*points.last().unwrap());

        self.solver_point_renderer.update_vertices(&points, queue);
        self.solver_line_renderer.update_lines(&lines, queue);
    }

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        self.solver_line_renderer.render(render_pass)?;
        self.solver_point_renderer.render(render_pass)?;
        Ok(())
    }
}
