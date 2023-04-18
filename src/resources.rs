use std::{
    fs,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use anyhow::*;
use cgmath::Vector3;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindGroupLayout, BindingResource, BufferUsages, Device,
    Queue,
};

use crate::{
    rendering::model::{self, ModelVertex},
    texture,
};

use crate::animation::track::DefaultConstructible;

fn get_path(file_name: &str) -> PathBuf {
    Path::new(env!("OUT_DIR")).join("res").join(file_name)
}

pub async fn load_string(file_name: &str) -> Result<String> {
    let txt = fs::read_to_string(get_path(file_name))?;
    Ok(txt)
}

pub async fn load_binary(file_name: &str) -> Result<Vec<u8>> {
    let data = fs::read(get_path(file_name))?;
    Ok(data)
}

pub async fn load_texture(
    file_name: &str,
    device: &Device,
    queue: &Queue,
) -> Result<texture::Texture> {
    let data = load_binary(file_name).await?;
    texture::Texture::from_bytes(device, queue, &data, file_name)
}

pub async fn load_model(
    file_name: &str,
    device: &Device,
    queue: &Queue,
    layout: &BindGroupLayout,
) -> Result<model::Model> {
    let text = load_string(file_name).await?;
    let cursor = Cursor::new(text);
    let reader = BufReader::new(cursor);
    match file_name
        .split(".")
        .last()
        .expect(&format!("Unknown file type {}", file_name))
    {
        "obj" => load_obj_model(file_name, device, queue, layout, reader).await,
        "gltf" => todo!(),
        _ => panic!("Unable to parse format {}", file_name),
    }
}

async fn load_obj_model(
    file_name: &str,
    device: &Device,
    queue: &Queue,
    layout: &BindGroupLayout,
    mut obj_reader: BufReader<Cursor<String>>,
) -> Result<model::Model> {
    let (models, obj_materials) = tobj::load_obj_buf_async(
        &mut obj_reader,
        &tobj::LoadOptions {
            single_index: true,
            triangulate: true,
            ..Default::default()
        },
        |p| async move {
            let mat_text = load_string(&p).await.unwrap();
            tobj::load_mtl_buf(&mut BufReader::new(Cursor::new(mat_text)))
        },
    )
    .await?;

    let mut materials = vec![];
    for m in obj_materials? {
        let diffuse_texture = load_texture(&m.diffuse_texture, device, queue).await?;
        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout,
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

        materials.push(model::Material {
            name: m.name,
            diffuse_texture,
            bind_group,
        })
    }

    let meshes = models
        .into_iter()
        .map(|m| {
            let vertices = (0..m.mesh.positions.len() / 3)
                .map(|i| ModelVertex {
                    position: [
                        m.mesh.positions[i * 3],
                        m.mesh.positions[i * 3 + 1],
                        m.mesh.positions[i * 3 + 2],
                    ],
                    tex_coords: [m.mesh.texcoords[i * 2], m.mesh.texcoords[i * 2 + 1]],
                    normal: [
                        m.mesh.normals[i * 3],
                        m.mesh.normals[i * 3 + 1],
                        m.mesh.normals[i * 3 + 2],
                    ],
                })
                .collect::<Vec<_>>();

            let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{:?} Vertex buffer", file_name)),
                contents: bytemuck::cast_slice(&vertices),
                usage: BufferUsages::VERTEX,
            });
            let index_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some(&format!("{:?} Index buffer", file_name)),
                contents: bytemuck::cast_slice(&m.mesh.indices),
                usage: BufferUsages::INDEX,
            });

            model::Mesh {
                name: file_name.to_string(),
                vertex_buffer,
                index_buffer,
                num_elements: m.mesh.indices.len() as u32,
                material: m.mesh.material_id.unwrap_or(0),
                model_vertices: vertices,
                positions: Vector3::default(),
            }
        })
        .collect::<Vec<_>>();

    Ok(model::Model { meshes, materials })
}
