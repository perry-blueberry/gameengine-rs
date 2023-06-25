use either::Either;
use gilrs::Axis;
use winit::event::{self, ElementState, KeyboardInput, WindowEvent};

use crate::camera::Camera;

pub struct CameraController {
    speed: f32,
    y: f32,
    x: f32,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            y: 0.0,
            x: 0.0,
        }
    }

    pub fn process_events(&mut self, event: Either<&WindowEvent, gilrs::Event>) -> bool {
        match event {
            Either::Left(event) => match event {
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(keycode),
                            ..
                        },
                    ..
                } => {
                    let is_pressed = *state == ElementState::Pressed;
                    match keycode {
                        event::VirtualKeyCode::Left => self.x = if is_pressed { -1.0 } else { 0.0 },
                        event::VirtualKeyCode::Up => self.y = if is_pressed { 1.0 } else { 0.0 },
                        event::VirtualKeyCode::Right => self.x = if is_pressed { 1.0 } else { 0.0 },
                        event::VirtualKeyCode::Down => self.y = if is_pressed { -1.0 } else { 0.0 },
                        _ => return false,
                    }
                    true
                }
                _ => false,
            },
            Either::Right(e) => match e.event {
                gilrs::EventType::AxisChanged(Axis::LeftStickX, v, _) if v.abs() < 0.05 => {
                    self.x = 0.0;
                    true
                }
                gilrs::EventType::AxisChanged(Axis::LeftStickX, v, _) => {
                    self.x = v;
                    true
                }
                gilrs::EventType::AxisChanged(Axis::LeftStickY, v, _) if v.abs() < 0.05 => {
                    self.y = 0.0;
                    true
                }
                gilrs::EventType::AxisChanged(Axis::LeftStickY, v, _) => {
                    self.y = v;
                    true
                }
                _ => false,
            },
        }
    }

    pub fn update_camera<C: Camera>(&self, camera: &mut C, delta_time: f32) {
        let forward = camera.target() - camera.eye();
        let forward_norm = forward.normalized();
        let forward_mag = forward.magnitude();

        let speed = self.speed * delta_time;

        if (self.y > 0.0 && forward_mag > speed) || self.y < 0.0 {
            camera.set_eye(camera.eye() + forward_norm * speed * self.y);
        }

        let right = forward_norm.cross(camera.up());
        let forward = camera.target() - camera.eye();
        let forward_mag = forward.magnitude();

        if self.x.abs() != 0.0 {
            camera.set_eye(
                camera.target() - (forward + right * speed * self.x).normalized() * forward_mag,
            );
        }
    }
}
