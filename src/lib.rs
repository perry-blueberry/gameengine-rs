use std::time::{Duration, Instant};

use rendering::state::State;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
};

pub mod animation;
pub mod camera;
mod camera_controller;
mod collisions;
mod instance;
mod math;
pub mod rendering;
mod resources;
mod texture;

pub fn run(event_loop: EventLoop<()>, mut state: State) {
    env_logger::init();

    let mut previous_time = Instant::now();
    let mut frame_counter = 0;
    let mut delta_accum = Duration::ZERO;
    event_loop.run(move |event, _, control_flow| {
        let now = Instant::now();
        let delta = now - previous_time;
        delta_accum += delta;
        previous_time = now;
        frame_counter += 1;
        if delta_accum.as_secs() > 1 {
            println!(
                "FPS: {}",
                1.0 / (delta_accum.as_secs_f32() / frame_counter as f32)
            );
            frame_counter = 0;
            delta_accum = Duration::ZERO;
        }
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    match event {
                        WindowEvent::CloseRequested
                        | WindowEvent::KeyboardInput {
                            input:
                                KeyboardInput {
                                    state: ElementState::Pressed,
                                    virtual_keycode: Some(VirtualKeyCode::Escape),
                                    ..
                                },
                            ..
                        } => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            state.resize(*physical_size);
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // new_inner_size is &&mut so w have to dereference it twice
                            state.resize(**new_inner_size);
                        }
                        _ => {}
                    }
                }
            }
            Event::RedrawRequested(window_id) if window_id == state.window().id() => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    // Reconfigure the surface if it's lost or outdated
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.size())
                    }
                    // The system is out of memory, we should probably quit
                    Err(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,

                    Err(wgpu::SurfaceError::Timeout) => log::warn!("Surface timeout"),
                }
            }
            Event::RedrawEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                state.window().request_redraw();
            }
            _ => {}
        }
    });
}
