use gameengine_rs::{
    animation::gltf_loader::load_rest_pose,
    rendering::{line::LineRender, renderable::Renderable, state::State},
    run,
};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let (document, buffers, images) = gltf::import("res/Woman.gltf").expect("Failed to open gltf");
    let rest_pose = load_rest_pose(&document);
    let mut state = pollster::block_on(State::new(window));
    let line_render = LineRender::new(
        lines,
        &state.device,
        &state.config,
        &state.camera_ortho_buffer,
    );
    state.add_ui_renderable(Renderable::Line(line_render));
    run(event_loop, state);
}
