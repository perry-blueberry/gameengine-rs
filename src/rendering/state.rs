use std::iter;

use crate::camera::{CameraOrtho, CameraPerspective, CameraUniform};
use crate::camera_controller::CameraController;

use crate::texture;
use bytemuck::cast_slice;
use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BufferUsages, LoadOp, Operations, RenderPassDepthStencilAttachment,
};
use wgpu::{Device, Queue, Surface, SurfaceConfiguration};
use winit::dpi::PhysicalSize;
use winit::event::*;

use winit::window::Window;

use super::renderable::{Renderable, RenderableT, Updatable};

pub struct State {
    pub surface: Surface,
    pub device: Device,
    pub queue: Queue,
    pub config: SurfaceConfiguration,
    size: PhysicalSize<u32>,
    window: Window,
    depth_texture: texture::Texture,
    camera_persp: CameraPerspective,
    camera_persp_uniform: CameraUniform,
    pub camera_persp_buffer: wgpu::Buffer,
    camera_persp_controller: CameraController,
    camera_ortho: CameraOrtho,
    camera_ortho_uniform: CameraUniform,
    pub camera_ortho_buffer: wgpu::Buffer,
    renderables: Vec<Renderable>,
    ui_renderables: Vec<Renderable>,
}

impl State {
    pub async fn new(window: Window) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            dx12_shader_compiler: Default::default(),
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = unsafe { instance.create_surface(&window) }.unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                    label: None,
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .filter(|f| f.describe().srgb)
            .next()
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };
        surface.configure(&device, &config);

        let depth_texture =
            texture::Texture::create_depth_texture(&device, &config, "depth_texture");

        let camera_persp = CameraPerspective {
            aspect: config.width as f32 / config.height as f32,
            eye: (0.0, 4.0, 7.0).into(),
            target: (0.0, 4.0, 0.0).into(),
            fovy: 60.0,
            znear: 0.1,
            zfar: 100.0,
            up: Vector3::unit_y(),
        };

        let camera_persp_controller = CameraController::new(0.02);

        let mut camera_persp_uniform: CameraUniform = CameraUniform::new();
        camera_persp_uniform.update_view_proj(&camera_persp);

        let camera_persp_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Perspective Camera Buffer"),
            contents: cast_slice(&[camera_persp_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let camera_ortho = CameraOrtho {
            eye: (0.0, 0.0, 5.0).into(),
            target: (0.0, 0.0, 0.0).into(),
            up: Vector3::unit_y(),
            left: 0.0,
            right: (config.width as f32 / config.height as f32) * 22.0,
            bottom: 0.0,
            top: 22.0,
            near: 0.001,
            far: 10.0,
        };

        let mut camera_ortho_uniform = CameraUniform::new();
        camera_ortho_uniform.update_view_proj(&camera_ortho);

        let camera_ortho_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Orthographic Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_ortho_uniform]),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            depth_texture,
            camera_persp,
            camera_persp_uniform,
            camera_persp_buffer,
            camera_persp_controller,
            camera_ortho,
            camera_ortho_uniform,
            camera_ortho_buffer,
            renderables: vec![],
            ui_renderables: vec![],
        }
    }

    pub fn add_renderable(&mut self, renderable: Renderable) {
        self.renderables.push(renderable);
    }

    pub fn add_ui_renderable(&mut self, renderable: Renderable) {
        self.ui_renderables.push(renderable);
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
        self.depth_texture =
            texture::Texture::create_depth_texture(&self.device, &self.config, "depth_texture");
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_persp_controller.process_events(event)
    }

    pub fn update(&mut self, delta_time: f32) {
        self.camera_persp_controller
            .update_camera(&mut self.camera_persp);
        self.camera_persp_uniform
            .update_view_proj(&self.camera_persp);
        self.queue.write_buffer(
            &self.camera_persp_buffer,
            0,
            cast_slice(&[self.camera_persp_uniform]),
        );
        for renderable in &mut self.renderables {
            renderable.update(delta_time, &self.queue);
        }
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        if !self.renderables.is_empty() {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: Some(RenderPassDepthStencilAttachment {
                    view: &self.depth_texture.view,
                    depth_ops: Some(Operations {
                        load: LoadOp::Clear(1.0),
                        store: true,
                    }),
                    stencil_ops: None,
                }),
            });
            for renderable in &mut self.renderables {
                renderable.render(&mut render_pass)?;
            }
        }

        if !self.ui_renderables.is_empty() {
            let mut ui_render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("UI Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });
            for ui_renderable in &mut self.ui_renderables {
                ui_renderable.render(&mut ui_render_pass)?;
            }
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    pub fn size(&self) -> PhysicalSize<u32> {
        self.size
    }
}
