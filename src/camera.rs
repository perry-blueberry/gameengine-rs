use bytemuck::{Pod, Zeroable};
use cgmath::{ortho, perspective, Deg, Matrix4, Point3, SquareMatrix, Vector3};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.0,
    0.0, 0.0, 0.5, 1.0,
);

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
    fn build_view_projection_matrix(&self) -> Matrix4<f32>;
    fn eye(&self) -> Point3<f32>;
    fn set_eye(&mut self, eye: Point3<f32>);
    fn target(&self) -> Point3<f32>;
    fn up(&self) -> Vector3<f32>;
}

pub struct CameraPerspective {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub aspect: f32,
    pub fovy: f32,
    pub znear: f32,
    pub zfar: f32,
}

impl Camera for CameraPerspective {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = perspective(Deg(self.fovy), self.aspect, self.znear, self.zfar);
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    fn eye(&self) -> Point3<f32> {
        self.eye
    }

    fn target(&self) -> Point3<f32> {
        self.target
    }

    fn set_eye(&mut self, eye: Point3<f32>) {
        self.eye = eye;
    }

    fn up(&self) -> Vector3<f32> {
        self.up
    }
}

pub struct CameraOrtho {
    pub eye: Point3<f32>,
    pub target: Point3<f32>,
    pub up: Vector3<f32>,
    pub left: f32,
    pub right: f32,
    pub bottom: f32,
    pub top: f32,
    pub near: f32,
    pub far: f32,
}

impl Camera for CameraOrtho {
    fn build_view_projection_matrix(&self) -> Matrix4<f32> {
        let view = Matrix4::look_at_rh(self.eye, self.target, self.up);
        let proj = ortho(
            self.left,
            self.right,
            self.bottom,
            self.top,
            self.near,
            self.far,
        );
        OPENGL_TO_WGPU_MATRIX * proj * view
    }

    fn eye(&self) -> Point3<f32> {
        self.eye
    }

    fn target(&self) -> Point3<f32> {
        self.target
    }

    fn set_eye(&mut self, eye: Point3<f32>) {
        self.eye = eye;
    }

    fn up(&self) -> Vector3<f32> {
        self.up
    }
}
