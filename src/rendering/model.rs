use crate::instance::InstanceRaw;
use crate::math::vector3::Vector3;
use crate::{instance::Instance, rendering::renderable::RenderableT, texture};
use anyhow::Ok;
use anyhow::Result;
use bytemuck::cast_slice;
use bytemuck::{Pod, Zeroable};
use std::sync::{Arc, RwLock};
use std::{mem::size_of, ops::Range};
use wgpu::util::BufferInitDescriptor;
use wgpu::util::DeviceExt;
use wgpu::{
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingType, BufferAddress, BufferBindingType, BufferUsages, ColorWrites,
    CompareFunction, DepthBiasState, DepthStencilState, Device, IndexFormat, MultisampleState,
    Queue, RenderPass, RenderPipeline, ShaderStages, StencilState, SurfaceConfiguration,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};
use wgpu::{BindGroupLayout, ShaderModule};

use super::renderable::Vertex;

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct ModelVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex for ModelVertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<ModelVertex>() as BufferAddress,
            step_mode: VertexStepMode::Vertex,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: 0,
                    shader_location: 0,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x2,
                    offset: size_of::<[f32; 3]>() as BufferAddress,
                    shader_location: 1,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x3,
                    offset: size_of::<[f32; 5]>() as BufferAddress,
                    shader_location: 2,
                },
            ],
        }
    }
}

#[derive(Debug)]
pub struct Mesh<T: Vertex> {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_elements: u32,
    pub material: usize,
    pub model_vertices: Vec<T>,
    pub positions: Vector3,
}

pub struct Material {
    pub name: String,
    pub diffuse_texture: Arc<RwLock<texture::Texture>>,
    pub bind_group: BindGroup,
}

pub struct Model<T: Vertex> {
    pub meshes: Vec<Mesh<T>>,
    pub materials: Vec<Material>,
}

pub trait DrawModel<'a, T: Vertex> {
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh<T>,
        material: &'a Material,
        bind_groups: &Vec<(u32, &'a BindGroup)>,
    );
    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh<T>,
        material: &'a Material,
        instances: Range<u32>,
        bind_groups: &Vec<(u32, &'a BindGroup)>,
    );

    fn draw_model(&mut self, model: &'a Model<T>, bind_groups: Vec<(u32, &'a BindGroup)>);
    fn draw_model_instanced(
        &mut self,
        model: &'a Model<T>,
        instances: Range<u32>,
        bind_groups: Vec<(u32, &'a BindGroup)>,
    );
}

impl<'a, 'b, T: Vertex> DrawModel<'b, T> for RenderPass<'a>
where
    'b: 'a,
{
    fn draw_mesh(
        &mut self,
        mesh: &'a Mesh<T>,
        material: &'a Material,
        bind_groups: &Vec<(u32, &'a BindGroup)>,
    ) {
        self.draw_mesh_instanced(mesh, material, 0..1, bind_groups);
    }

    fn draw_mesh_instanced(
        &mut self,
        mesh: &'a Mesh<T>,
        material: &'a Material,
        instances: Range<u32>,
        bind_groups: &Vec<(u32, &'a BindGroup)>,
    ) {
        self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
        self.set_index_buffer(mesh.index_buffer.slice(..), IndexFormat::Uint32);
        self.set_bind_group(0, &material.bind_group, &[]);
        for (group_index, bind_group) in bind_groups {
            self.set_bind_group(*group_index, *bind_group, &[]);
        }
        self.draw_indexed(0..mesh.num_elements, 0, instances);
    }

    fn draw_model(&mut self, model: &'a Model<T>, bind_groups: Vec<(u32, &'a BindGroup)>) {
        self.draw_model_instanced(model, 0..1, bind_groups);
    }

    fn draw_model_instanced(
        &mut self,
        model: &'a Model<T>,
        instances: Range<u32>,
        bind_groups: Vec<(u32, &'a BindGroup)>,
    ) {
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material];
            self.draw_mesh_instanced(mesh, material, instances.clone(), &bind_groups);
        }
    }
}

pub struct TriangleModel {
    render_pipeline: RenderPipeline,
    instances: Vec<Instance>,
    instance_buffer: wgpu::Buffer,
    model: Model<ModelVertex>,
    camera_bind_group: BindGroup,
}

pub async fn new_model(
    model: Model<ModelVertex>,
    texture_bind_group_layout: BindGroupLayout,
    instances: Vec<Instance>,
    device: &Device,
    queue: &Queue,
    config: &SurfaceConfiguration,
    camera_buffer: &wgpu::Buffer,
    shader: ShaderModule,
) -> Result<(
    RenderPipeline,
    Vec<Instance>,
    wgpu::Buffer,
    Model<ModelVertex>,
    BindGroup,
)> {
    let camera_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
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

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout"),
        bind_group_layouts: &[&texture_bind_group_layout, &camera_bind_group_layout],
        push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[ModelVertex::desc(), InstanceRaw::desc()],
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
        depth_stencil: Some(DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
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

    let instance_data = instances.iter().map(Instance::to_raw).collect::<Vec<_>>();
    let instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("instance_buffer"),
        contents: cast_slice(&instance_data),
        usage: BufferUsages::VERTEX,
    });
    Ok((
        render_pipeline,
        instances,
        instance_buffer,
        model,
        camera_bind_group,
    ))
}

impl TriangleModel {
    pub async fn new(
        model: Model<ModelVertex>,
        texture_bind_group_layout: BindGroupLayout,
        instances: Vec<Instance>,
        device: &Device,
        queue: &Queue,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
    ) -> Result<Self> {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));
        let (render_pipeline, instances, instance_buffer, model, camera_bind_group) = new_model(
            model,
            texture_bind_group_layout,
            instances,
            device,
            queue,
            config,
            camera_buffer,
            shader,
        )
        .await?;
        Ok(Self {
            render_pipeline,
            instances,
            instance_buffer,
            model,
            camera_bind_group,
        })
    }
}

impl RenderableT for TriangleModel {
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update(&mut self, _delta_time: f32, _queue: &Queue) {}

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_model_instanced(
            &self.model,
            0..self.instances.len() as u32,
            vec![(1, &self.camera_bind_group)],
        );
        std::result::Result::Ok(())
    }
}
