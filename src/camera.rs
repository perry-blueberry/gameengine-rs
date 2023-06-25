use bytemuck::{Pod, Zeroable};

use math::{matrix4::Matrix4, vector3::Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4 = Matrix4{values: [
    [1.0, 0.0, 0.0, 0.0,],
    [0.0, 1.0, 0.0, 0.0,],
    [0.0, 0.0, 0.5, 0.0,],
    [0.0, 0.0, 0.5, 1.0,]]};

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    pub fn new() -> Self {
        // SquareMatrix
        Self {
            view_proj: Matrix4::identity().into(),
        }
    }

    pub fn update_view_proj<C: Camera>(&mut self, camera: &C) {
        self.view_proj = camera.build_view_projection_matrix().into();
    }
}

pub trait Camera {
    fn build_view_projection_matrix(&self) -> Matrix4;
    fn eye(&self) -> Vector3;
    fn set_eye(&mut self, eye: Vector3);
    fn target(&self) -> Vector3;
    fn up(&self) -> Vector3;
}

pub struct CameraPerspective {
    pub eye: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera for CameraPerspective {
    fn build_view_projection_matrix(&self) -> Matrix4 {
        let view = Matrix4::look_at(self.eye, self.target, self.up).expect(&format!(
            "Failed to look at: eye {:?}, target {:?}, up {:?}",
            self.eye, self.target, self.up
        ));
        let proj = Matrix4::perspective(self.fovy, self.aspect, self.znear, self.zfar);
        &(&OPENGL_TO_WGPU_MATRIX * &proj) * &view
    }

    fn eye(&self) -> Vector3 {
        self.eye
    }

    fn target(&self) -> Vector3 {
        self.target
    }

    fn set_eye(&mut self, eye: Vector3) {
        self.eye = eye;
    }

    fn up(&self) -> Vector3 {
        self.up
    }
}

pub struct CameraOrtho {
    pub eye: Vector3,
    pub target: Vector3,
    pub up: Vector3,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera for CameraOrtho {
    fn build_view_projection_matrix(&self) -> Matrix4 {
        let view = Matrix4::look_at(self.eye, self.target, self.up).expect(&format!(
            "Failed to look at: eye {:?}, target {:?}, up {:?}",
            self.eye, self.target, self.up
        ));
        let proj = Matrix4::ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        &(&OPENGL_TO_WGPU_MATRIX * &proj) * &view
    }

    fn eye(&self) -> Vector3 {
        self.eye
    }

    fn target(&self) -> Vector3 {
        self.target
    }

    fn set_eye(&mut self, eye: Vector3) {
        self.eye = eye;
    }

    fn up(&self) -> Vector3 {
        self.up
    }
}
