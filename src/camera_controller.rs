use winit::event::{self, ElementState, KeyboardInput, WindowEvent};

use crate::camera::Camera;

pub struct CameraController {
    speed: f32,
    is_forward_pressed: bool,
    is_backward_pressed: bool,
    is_left_pressed: bool,
    is_right_pressed: bool,
}

impl CameraController {
    pub fn new(speed: f32) -> Self {
        Self {
            speed,
            is_forward_pressed: false,
            is_backward_pressed: false,
            is_left_pressed: false,
            is_right_pressed: false,
        }
    }

    pub fn process_events(&mut self, event: &WindowEvent) -> bool {
        match event {
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
                    event::VirtualKeyCode::Left => self.is_left_pressed = is_pressed,
                    event::VirtualKeyCode::Up => self.is_forward_pressed = is_pressed,
                    event::VirtualKeyCode::Right => self.is_right_pressed = is_pressed,
                    event::VirtualKeyCode::Down => self.is_backward_pressed = is_pressed,
                    _ => return false,
                }
                true
            }
            _ => false,
        }
    }

    pub fn update_camera<C: Camera>(&self, camera: &mut C) {
        let forward = camera.target() - camera.eye();
        let forward_norm = forward.normalized();
        let forward_mag = forward.magnitude();

        if self.is_forward_pressed && forward_mag > self.speed {
            camera.set_eye(camera.eye() + forward_norm * self.speed);
        }
        if self.is_backward_pressed {
            camera.set_eye(camera.eye() - forward_norm * self.speed);
        }

        let right = forward_norm.cross(camera.up());
        let forward = camera.target() - camera.eye();
        let forward_mag = forward.magnitude();

        if self.is_right_pressed {
            camera.set_eye(
                camera.target() - (forward + right * self.speed).normalized() * forward_mag,
            );
        }
        if self.is_left_pressed {
            camera.set_eye(
                camera.target() - (forward - right * self.speed).normalized() * forward_mag,
            );
        }
    }
}
