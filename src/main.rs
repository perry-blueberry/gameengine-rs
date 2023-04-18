use gameengine_rs::{
    rendering::{model, renderable::Renderable, state::State},
    run,
};
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(window));
    let model = pollster::block_on(model::TriangleModel::new(
        "cube.obj",
        &state.device,
        &state.queue,
        &state.config,
        &state.camera_persp_buffer,
    ))
    .expect("Unable to create model");
    state.add_renderable(Renderable::Model(model));
    run(event_loop, state);
}
