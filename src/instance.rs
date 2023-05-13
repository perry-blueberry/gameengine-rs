use std::mem::size_of;

use bytemuck::{Pod, Zeroable};
use num_traits::Zero;
use wgpu::{BufferAddress, VertexAttribute, VertexBufferLayout, VertexFormat, VertexStepMode};

use crate::math::{matrix4::Matrix4, quaternion::Quaternion, vector3::Vector3};

pub struct Instance {
    pub position: Vector3,
    pub rotation: Quaternion,
}

impl Instance {
    pub fn to_raw(&self) -> InstanceRaw {
        InstanceRaw {
            model: (&Matrix4::from_translation(self.position) * &Matrix4::from(self.rotation))
                .into(),
        }
    }
}

const NUM_INSTANCES_PER_ROW: u32 = 10;
const SPACE_BETWEEN_INSTANCE: f32 = 3.0;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct InstanceRaw {
    model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc<'a>() -> VertexBufferLayout<'a> {
        VertexBufferLayout {
            array_stride: size_of::<InstanceRaw>() as BufferAddress,
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: 0,
                    shader_location: 5,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: size_of::<[f32; 4]>() as BufferAddress,
                    shader_location: 6,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: size_of::<[f32; 8]>() as BufferAddress,
                    shader_location: 7,
                },
                VertexAttribute {
                    format: VertexFormat::Float32x4,
                    offset: size_of::<[f32; 12]>() as BufferAddress,
                    shader_location: 8,
                },
            ],
        }
    }
}

pub fn create_instances() -> Vec<Instance> {
    (0..NUM_INSTANCES_PER_ROW)
        .flat_map(|z| {
            (0..NUM_INSTANCES_PER_ROW).map(move |x| {
                let x = SPACE_BETWEEN_INSTANCE * (x as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let z = SPACE_BETWEEN_INSTANCE * (z as f32 - NUM_INSTANCES_PER_ROW as f32 / 2.0);
                let position = Vector3 { x, y: 0.0, z };

                let rotation = if position.is_zero() {
                    Quaternion::from_axis_angle(
                        Vector3 {
                            x: 0.0,
                            y: 0.0,
                            z: 1.0,
                        },
                        0.0,
                    )
                } else {
                    Quaternion::from_axis_angle(position.normalized(), 45.0)
                };

                Instance { position, rotation }
            })
        })
        .collect::<Vec<_>>()
}
