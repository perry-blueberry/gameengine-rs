use std::iter;

use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    vertex_attr_array, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferAddress, BufferUsages, ColorWrites,
    IndexFormat, MultisampleState, SamplerBindingType, ShaderStages, TextureSampleType,
    TextureViewDimension, VertexAttribute, VertexBufferLayout,
};
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};

// lib.rs
use winit::window::Window;

mod texture;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [VertexAttribute; 2] = vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

const VERTICES: &[Vertex] = &[
    Vertex {
        position: [-0.0868241, 0.49240386, 0.0],
        tex_coords: [0.4131759, 0.00759614],
    },
    Vertex {
        position: [-0.49513406, 0.06958647, 0.0],
        tex_coords: [0.0048659444, 0.43041354],
    },
    Vertex {
        position: [-0.21918549, -0.44939706, 0.0],
        tex_coords: [0.28081453, 0.949397],
    },
    Vertex {
        position: [0.35966998, -0.3473291, 0.0],
        tex_coords: [0.85967, 0.84732914],
    },
    Vertex {
        position: [0.44147372, 0.2347359, 0.0],
        tex_coords: [0.9414737, 0.2652641],
    },
];

const INDICES: &[u16] = &[0, 1, 4, 1, 2, 4, 2, 3, 4];

struct State {
    surface: wgpu::Surface,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    window: Window,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    diffuse_bind_group: BindGroup,
    diffuse_texture: texture::Texture,
}

impl State {
    // Creating some of the wgpu types requires async code
    async fn new(window: Window) -> Self {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // Backends::all => Vulkan + Metal + DX12 + Browser WebGPU
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
        // Shader code in this tutorial assumes an sRGB surface texture. Using a different
        // one will result all the colors coming out darker. If you want to support non
        // sRGB surfaces, you'll need to account for that when drawing to the frame.
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

        let diffuse_bytes = include_bytes!("img/happy-tree.png");
        let diffuse_texture =
            texture::Texture::from_bytes(&device, &queue, diffuse_bytes, "happy-tree.png").unwrap();

        let texture_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("texture_bind_group_layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        count: None,
                        ty: BindingType::Texture {
                            sample_type: TextureSampleType::Float { filterable: true },
                            view_dimension: TextureViewDimension::D2,
                            multisampled: false,
                        },
                        visibility: ShaderStages::FRAGMENT,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        count: None,
                        ty: BindingType::Sampler(SamplerBindingType::Filtering),
                        visibility: ShaderStages::FRAGMENT,
                    },
                ],
            });
        let diffuse_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("diffuse_bind_group"),
            layout: &&texture_bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&diffuse_texture.view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&diffuse_texture.sampler),
                },
            ],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render pipeline layout"),
                bind_group_layouts: &[&texture_bind_group_layout],
                push_constant_ranges: &[],
            });
        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: ColorWrites::all(),
                })],
            }),
            multiview: None,
        });

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: BufferUsages::VERTEX,
        });
        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: BufferUsages::INDEX,
        });

        let num_indices = INDICES.len() as u32;
        Self {
            window,
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            diffuse_bind_group,
            diffuse_texture,
        }
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
    }

    #[allow(unused_variables)]
    fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
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
                depth_stencil_attachment: None,
            });
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.diffuse_bind_group, &[]);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint16);
            render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

pub async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = State::new(window).await;

    event_loop.run(move |event, _, control_flow| {
        match event {
            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == state.window().id() => {
                if !state.input(event) {
                    // UPDATED!
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
                        state.resize(state.size)
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
