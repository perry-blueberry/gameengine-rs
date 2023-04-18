use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferBindingType, BufferUsages, Device, IndexFormat,
    RenderPipeline, ShaderStages, SurfaceConfiguration,
};

use super::renderable::{RenderableT, SimpleVertex, Vertex};

pub struct PointRender {
    render_pipeline: RenderPipeline,
    camera_bind_group: BindGroup,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
}

impl PointRender {
    pub fn new(
        vertices: &Vec<SimpleVertex>,
        device: &Device,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
    ) -> Self {
        let camera_bind_group_layout =
            device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("camera_bind_group_layout"),
                entries: &[BindGroupLayoutEntry {
                    binding: 0,
                    count: None,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: ShaderStages::VERTEX,
                }],
            });
        let camera_bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("camera_bind_group"),
            layout: &camera_bind_group_layout,
            entries: &[BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("line.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[SimpleVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::REPLACE,
                        alpha: wgpu::BlendComponent::REPLACE,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
                // or Features::POLYGON_MODE_POINT
                polygon_mode: wgpu::PolygonMode::Fill,
                // Requires Features::DEPTH_CLIP_CONTROL
                unclipped_depth: false,
                // Requires Features::CONSERVATIVE_RASTERIZATION
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let (vertices, indices) = vertices_to_points(vertices);

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: BufferUsages::INDEX,
        });

        Self {
            render_pipeline,
            camera_bind_group,
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
        }
    }
}

impl RenderableT for PointRender {
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update(&mut self) {}

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        render_pass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
        render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
        Ok(())
    }
}

const POINT_SIZE: f32 = 0.1;

fn vertices_to_points(vertices: &Vec<SimpleVertex>) -> (Vec<SimpleVertex>, Vec<u32>) {
    let mut new_vertices = Vec::new();
    /* new_vertices.reserve(vertices.len() * 4); */
    let mut new_indices = Vec::new();
    /* new_indices.reserve(0); */
    for (idx, vertex) in vertices.iter().enumerate() {
        let start_index = new_vertices.len() as u32;
        println!("start_index {}", start_index);
        new_vertices.push(SimpleVertex {
            position: [
                vertex.position[0] - POINT_SIZE,
                vertex.position[1] - POINT_SIZE,
                vertex.position[2],
            ],
            color: vertex.color,
        });
        new_vertices.push(SimpleVertex {
            position: [
                vertex.position[0] - POINT_SIZE,
                vertex.position[1] + POINT_SIZE,
                vertex.position[2],
            ],
            color: vertex.color,
        });
        new_vertices.push(SimpleVertex {
            position: [
                vertex.position[0] + POINT_SIZE,
                vertex.position[1] + POINT_SIZE,
                vertex.position[2],
            ],
            color: vertex.color,
        });
        new_vertices.push(SimpleVertex {
            position: [
                vertex.position[0] + POINT_SIZE,
                vertex.position[1] - POINT_SIZE,
                vertex.position[2],
            ],
            color: vertex.color,
        });

        new_indices.push(start_index);
        new_indices.push(start_index + 1);
        new_indices.push(start_index + 2);

        new_indices.push(start_index + 3);
        new_indices.push(start_index);
        new_indices.push(start_index + 2);
    }
    (new_vertices, new_indices)
}
