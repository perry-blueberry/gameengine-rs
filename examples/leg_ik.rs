use std::sync::{Arc, RwLock};

use gameengine_rs::{
    animation::{
        clip::Clip,
        frame::Frame,
        gltf_loader::{
            load_animation_clips, load_skeleton, load_skinned_meshes, load_static_meshes,
        },
        ik_leg::IkLeg,
        interpolation::Interpolation,
        track::{ScalarTrack, Vector3Track},
    },
    collisions::triangle_ray::mesh_to_triangles,
    instance::Instance,
    math::{quaternion::Quaternion, vector3::Vector3},
    rendering::{
        model::{self, Material, Model},
        render_players::ik_leg_player::IkLegPlayer,
        renderable::Renderable,
        skeletal_model::SkeletalModel,
        state::State,
    },
    resources::load_texture,
    run,
    texture::create_texture_bind_group_layout,
};
use glam::Vec3;
use num_traits::Zero;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt},
    BindGroupDescriptor, BindGroupEntry, BindingResource, BufferUsages,
};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(window));

    let (env_doc, env_buf, _) = gltf::import("res/IKCourse.gltf").expect("Failed to open gltf");
    let env_diffuse_texture =
        pollster::block_on(load_texture("uv.png", &state.device, &state.queue))
            .expect("Failed to read diffuse texture");
    let (vertices, indices, _material) = load_static_meshes(&env_doc, &env_buf);
    let env_diffuse_texture = Arc::new(RwLock::new(env_diffuse_texture));
    let texture_bind_group_layout = create_texture_bind_group_layout(&state.device);

    let bind_group = state.device.create_bind_group(&BindGroupDescriptor {
        label: None,
        layout: &texture_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: BindingResource::TextureView(&env_diffuse_texture.read().unwrap().view),
            },
            BindGroupEntry {
                binding: 1,
                resource: BindingResource::Sampler(&env_diffuse_texture.read().unwrap().sampler),
            },
        ],
    });
    let vertex_buffer = state.device.create_buffer_init(&BufferInitDescriptor {
        label: Some(&format!("{:?} Vertex buffer", "IKCourse.gltf")),
        contents: bytemuck::cast_slice(&vertices),
        usage: BufferUsages::VERTEX,
    });
    let index_buffer = state.device.create_buffer_init(&BufferInitDescriptor {
        label: Some(&format!("{:?} Index buffer", "IKCourse.gltf")),
        contents: bytemuck::cast_slice(&indices),
        usage: BufferUsages::INDEX,
    });
    let model = Model {
        meshes: vec![model::Mesh {
            name: "IKCourse.gltf".into(),
            vertex_buffer,
            index_buffer,
            num_elements: indices.len() as u32,
            material: 0,
            model_vertices: vertices.clone(),
            positions: Vector3::zero(),
        }],
        materials: vec![Material {
            name: "uv.png".into(),
            diffuse_texture: env_diffuse_texture,
            bind_group,
        }],
    };
    let model = pollster::block_on(model::TriangleModel::new(
        model,
        texture_bind_group_layout,
        vec![Instance {
            position: Vector3::zero(),
            rotation: Quaternion::default(),
        }],
        &state.device,
        &state.queue,
        &state.config,
        &state.camera_persp_buffer,
    ))
    .expect("Unable to create model");
    state.add_renderable(Renderable::Model(model));
    let triangles = mesh_to_triangles(&vertices, &indices);

    let (document, buffers, _images) = gltf::import("res/Woman.gltf").expect("Failed to open gltf");
    let (vertices, original_positions, original_normals, indices, material) =
        load_skinned_meshes(&document, &buffers);
    let diffuse_texture =
        pollster::block_on(load_texture("Woman.png", &state.device, &state.queue))
            .expect("Failed to read diffuse texture");
    let diffuse_texture = Arc::new(RwLock::new(diffuse_texture));
    let animation_clips: Vec<Clip> = load_animation_clips(&document, &buffers);
    let current_clip = animation_clips
        .iter()
        .find(|c| c.name == "Walking")
        .unwrap()
        .to_owned();
    let skeleton = load_skeleton(&document, &buffers);
    let instances = Arc::new(RwLock::new(vec![Instance {
        position: Vector3 {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Quaternion::default(),
    }]));

    let model = pollster::block_on(SkeletalModel::new(
        vertices,
        original_positions,
        original_normals,
        indices,
        "Woman.gltf",
        &state.device,
        &state.config,
        &state.camera_persp_buffer,
        material,
        diffuse_texture,
        current_clip.clone(),
        skeleton.clone(),
        instances,
    ))
    .unwrap();

    let mut left_leg = IkLeg::new(
        "LeftUpLeg",
        "LeftLeg",
        "LeftFoot",
        "LeftToeBase",
        0.2,
        &skeleton,
    );
    let left_track = &mut left_leg.pin_track;
    let frames = vec![
        Frame::new_simple(0.0, 0.0),
        Frame::new_simple(0.4, 1.0),
        Frame::new_simple(0.6, 1.0),
        Frame::new_simple(1.0, 0.0),
    ];
    *left_track = ScalarTrack::new_with_args(Interpolation::Cubic, frames);
    let mut right_leg = IkLeg::new(
        "RightUpLeg",
        "RightLeg",
        "RightFoot",
        "RightToeBase",
        0.2,
        &skeleton,
    );
    let right_track = &mut right_leg.pin_track;
    let frames = vec![
        Frame::new_simple(0.0, 1.0),
        Frame::new_simple(0.3, 0.0),
        Frame::new_simple(0.7, 0.0),
        Frame::new_simple(1.0, 1.0),
    ];
    *right_track = ScalarTrack::new_with_args(Interpolation::Cubic, frames);
    let frames = vec![
        Frame::new_simple(0.0, Vec3::new(0.0, 0.0, 1.0)),
        Frame::new_simple(1.0, Vec3::new(0.0, 0.0, 10.0)),
        Frame::new_simple(3.0, Vec3::new(22.0, 0.0, 10.0)),
        Frame::new_simple(4.0, Vec3::new(22.0, 0.0, 2.0)),
        Frame::new_simple(6.0, Vec3::new(0.0, 0.0, 1.0)),
    ];
    let motion_track = Vector3Track::new_with_args(Interpolation::Linear, frames);

    let ik_leg_player = IkLegPlayer::new(
        motion_track,
        triangles,
        0.15,
        current_clip,
        skeleton,
        left_leg,
        right_leg,
        0.3,
        model,
    );
    state.add_renderable(Renderable::IkLegPlayer(ik_leg_player));
    run(event_loop, state);
}
