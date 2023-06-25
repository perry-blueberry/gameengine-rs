use crate::{
    instance::{Instance, InstanceRaw},
    texture::{self, create_texture_bind_group_layout},
};
use animation::{clip::Clip, pose::Pose, skeleton::Skeleton};
use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use gltf::Material;
use math::{matrix4::Matrix4, vector3::Vector3};
use num_traits::Zero;
use std::{
    mem::size_of,
    sync::{Arc, RwLock},
};
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor,
    BindGroupLayoutEntry, BindingResource, BindingType, BufferAddress, BufferBindingType,
    BufferUsages, ColorWrites, CompareFunction, DepthBiasState, DepthStencilState, Device,
    MultisampleState, RenderPipeline, ShaderStages, StencilState, SurfaceConfiguration,
    VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode,
};

use super::{
    model::{self, DrawModel, Mesh, Model},
    renderable::{RenderableT, Vertex},
};
use anyhow::{Ok, Result};

#[repr(C)]
#[derive(Debug, Copy, Clone, Pod, Zeroable)]
pub struct SkeletalVertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
    pub weights: [f32; 4],
    pub joints: [u16; 4],
}

impl Vertex for SkeletalVertex {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<SkeletalVertex>() as BufferAddress,
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
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 3,
                },
                VertexAttribute {
                    format: VertexFormat::Uint16x4,
                    offset: size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 4,
                },
            ],
        }
    }
}

pub struct SkeletalModel {
    render_pipeline: RenderPipeline,
    model: Model<SkeletalVertex>,
    camera_bind_group: BindGroup,
    pose_bind_group: BindGroup,
    original_positions: Vec<[f32; 3]>,
    original_normals: Vec<[f32; 3]>,
    pub instance_buffer: wgpu::Buffer,
    pub animated_buffer: wgpu::Buffer,
    animated_pose: Pose,
    clip: Clip,
    skeleton: Skeleton,
    playback_time: f32,
}

impl SkeletalModel {
    pub async fn new<'a>(
        vertices: Vec<SkeletalVertex>,
        original_positions: Vec<[f32; 3]>,
        original_normals: Vec<[f32; 3]>,
        indices: Vec<u32>,
        model_name: &str,
        device: &Device,
        config: &SurfaceConfiguration,
        camera_buffer: &wgpu::Buffer,
        material: Material<'a>,
        diffuse_texture: Arc<RwLock<texture::Texture>>,
        clip: Clip,
        skeleton: Skeleton,
        instances: Arc<RwLock<Vec<Instance>>>,
    ) -> Result<Self> {
        let SkeletalModelBase {
            render_pipeline,
            model,
            camera_bind_group,
            pose_bind_group,
            original_positions,
            original_normals,
            instance_buffer,
            animated_buffer,
        } = {
            let instances = instances.read().unwrap();
            new_skeletal_pipeline(
                vertices,
                original_positions,
                original_normals,
                indices,
                model_name,
                device,
                config,
                camera_buffer,
                &material,
                diffuse_texture,
                &instances,
            )
        };

        Ok(Self {
            render_pipeline,
            model,
            camera_bind_group,
            pose_bind_group,
            original_positions,
            original_normals,
            instance_buffer,
            animated_buffer,
            animated_pose: skeleton.rest_pose.clone(),
            clip,
            skeleton,
            playback_time: 0.0,
        })
    }

    pub fn cpu_skin(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        if self.model.meshes.is_empty() {
            return;
        }
        let time = self.playback_time + delta_time;
        self.playback_time = self.clip.sample(&mut self.animated_pose, time);

        let pose_palette = self.animated_pose.matrix_palette();

        for (i, vertex) in &mut self.model.meshes[0].model_vertices.iter_mut().enumerate() {
            let j = vertex.joints;
            let w = vertex.weights;

            let m0 = (pose_palette[j[0] as usize] * self.skeleton.inverse_bind_pose[j[0] as usize])
                * w[0];
            let m1 = (pose_palette[j[1] as usize] * self.skeleton.inverse_bind_pose[j[1] as usize])
                * w[1];
            let m2 = (pose_palette[j[2] as usize] * self.skeleton.inverse_bind_pose[j[2] as usize])
                * w[2];
            let m3 = (pose_palette[j[3] as usize] * self.skeleton.inverse_bind_pose[j[3] as usize])
                * w[3];

            let skin = m0 + m1 + m2 + m3;
            vertex.position = skin
                .transform_point3(self.original_positions[i].into())
                .into();
            vertex.normal = skin
                .transform_vector3(self.original_normals[i].into())
                .into();
        }
        queue.write_buffer(
            &self.model.meshes[0].vertex_buffer,
            0,
            bytemuck::cast_slice(&self.model.meshes[0].model_vertices),
        );
    }

    fn gpu_skin(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        let time = self.playback_time + delta_time;
        self.playback_time = self.clip.sample(&mut self.animated_pose, time);
        let mut pose_palette: Vec<Mat4> = self.animated_pose.matrix_palette();
        for (i, p) in pose_palette.iter_mut().enumerate() {
            *p = *p * self.skeleton.inverse_bind_pose()[i];
        }
        queue.write_buffer(
            &self.animated_buffer,
            0,
            bytemuck::cast_slice(&pose_palette),
        );
    }
}

impl RenderableT for SkeletalModel {
    fn resize(&mut self, _new_size: winit::dpi::PhysicalSize<u32>) {}

    fn input(&mut self, _event: &winit::event::WindowEvent) -> bool {
        false
    }

    fn update(&mut self, delta_time: f32, queue: &wgpu::Queue) {
        self.gpu_skin(delta_time, queue);
    }

    fn render<'a, 'b: 'a>(
        &'b mut self,
        render_pass: &'a mut wgpu::RenderPass<'b>,
    ) -> Result<(), wgpu::SurfaceError> {
        render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        render_pass.set_pipeline(&self.render_pipeline);
        render_pass.draw_model_instanced(
            &self.model,
            0..1,
            vec![(1, &self.camera_bind_group), (2, &self.pose_bind_group)],
        );
        std::result::Result::Ok(())
    }
}

pub struct SkeletalModelBase {
    pub render_pipeline: RenderPipeline,
    pub model: Model<SkeletalVertex>,
    pub camera_bind_group: BindGroup,
    pub pose_bind_group: BindGroup,
    pub original_positions: Vec<[f32; 3]>,
    pub original_normals: Vec<[f32; 3]>,
    pub instance_buffer: wgpu::Buffer,
    pub animated_buffer: wgpu::Buffer,
}

pub fn new_skeletal_pipeline<'a>(
    vertices: Vec<SkeletalVertex>,
    original_positions: Vec<[f32; 3]>,
    original_normals: Vec<[f32; 3]>,
    indices: Vec<u32>,
    model_name: &str,
    device: &Device,
    config: &SurfaceConfiguration,
    camera_buffer: &wgpu::Buffer,
    material: &Material<'a>,
    diffuse_texture: Arc<RwLock<texture::Texture>>,
    instances: &Vec<Instance>,
) -> SkeletalModelBase {
    let shader = device.create_shader_module(wgpu::include_wgsl!("skeletal_model.wgsl"));
    let animated_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some("animated_buffer"),
        contents: bytemuck::cast_slice(&[Matrix4::identity(); 120]),
        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
    });
    let texture_bind_group_layout = create_texture_bind_group_layout(&device);

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

    let pose_bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
        label: Some("pose_bind_group_layout"),
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
    let pose_bind_group = device.create_bind_group(&BindGroupDescriptor {
        label: Some("pose_bind_group"),
        layout: &pose_bind_group_layout,
        entries: &[BindGroupEntry {
            binding: 0,
            resource: animated_buffer.as_entire_binding(),
        }],
    });

    let render_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("Render pipeline layout"),
        bind_group_layouts: &[
            &texture_bind_group_layout,
            &camera_bind_group_layout,
            &pose_bind_group_layout,
        ],
        push_constant_ranges: &[],
    });
    let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("Render pipeline"),
        layout: Some(&render_pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: "vs_main",
            buffers: &[SkeletalVertex::desc(), InstanceRaw::desc()],
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
        contents: bytemuck::cast_slice(&instance_data),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });
    let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some(&format!("{:?} Vertex buffer", model_name)),
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
    });
    let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
        label: Some(&format!("{:?} Index buffer", model_name)),
        contents: bytemuck::cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });
    let bind_group = {
        let diffuse_texture = diffuse_texture.read().unwrap();
        device.create_bind_group(&BindGroupDescriptor {
            label: Some("texture_bind_group"),
            layout: &texture_bind_group_layout,
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
        })
    };
    let model = Model {
        meshes: vec![Mesh {
            name: model_name.into(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
            model_vertices: vertices,
            positions: Vector3::zero(),
        }],
        materials: vec![model::Material {
            name: material.name().unwrap().into(),
            diffuse_texture,
            bind_group,
        }],
    };
    SkeletalModelBase {
        render_pipeline,
        model,
        camera_bind_group,
        pose_bind_group,
        original_positions,
        original_normals,
        instance_buffer,
        animated_buffer,
    }
}
