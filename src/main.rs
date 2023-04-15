use gameengine_rs::{rendering::state::State, run};
use winit::{event_loop::EventLoop, window::WindowBuilder};

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let state = pollster::block_on(State::new(window));
    run(event_loop, state);
}
