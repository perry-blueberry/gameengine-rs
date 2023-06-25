use gameengine_rs::state::State;
use gameengine_rs::{resources, run};
use rendering::instance::create_instances;
use rendering::texture::create_texture_bind_group_layout;
use rendering::{model, renderable::Renderable};
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(window));
    let texture_bind_group_layout = create_texture_bind_group_layout(&state.device);
    let model = pollster::block_on(resources::load_model(
        "cube.obj",
        &state.device,
        &state.queue,
        &texture_bind_group_layout,
    ))
    .unwrap();
    let instances = create_instances();
    let model = pollster::block_on(model::TriangleModel::new(
        model,
        texture_bind_group_layout,
        instances,
        &state.device,
        &state.queue,
        &state.config,
        &state.camera_persp_buffer,
    ))
    .expect("Unable to create model");
    state.add_renderable(Renderable::Model(model));
    run(event_loop, state);
}
