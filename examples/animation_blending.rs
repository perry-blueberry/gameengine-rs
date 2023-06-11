use std::sync::{Arc, RwLock};

use gameengine_rs::{
    animation::{
        gltf_loader::{load_animation_clips, load_skeleton, load_skinned_meshes},
        pose::Pose,
    },
    instance::Instance,
    math::{quaternion::Quaternion, vector3::Vector3},
    rendering::{
        render_players::blender_player::BlenderPlayer, renderable::Renderable, state::State,
    },
    resources::load_texture,
    run,
};

use winit::{event_loop::EventLoop, window::WindowBuilder};
pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let (document, buffers, _images) = gltf::import("res/Woman.gltf").expect("Failed to open gltf");
    let (vertices, original_positions, original_normals, indices, material) =
        load_skinned_meshes(&document, &buffers);
    let skeleton = Arc::new(load_skeleton(&document, &buffers));

    let (animation_clips, additive_index) = {
        let mut animation_clips = load_animation_clips(&document, &buffers);
        let additive_index = animation_clips
            .iter()
            .position(|c| c.name == "Lean_Left")
            .unwrap()
            .to_owned();
        animation_clips[additive_index].looping = false;
        (animation_clips, additive_index)
    };
    let clip_a = animation_clips
        .iter()
        .find(|c| c.name == "Walking")
        .unwrap()
        .to_owned();
    let time_a = clip_a.start_time;
    let clip_b = animation_clips
        .iter()
        .find(|c| c.name == "Running")
        .unwrap()
        .to_owned();
    let time_b = clip_b.start_time;
    let current_pose = skeleton.rest_pose.clone();
    let pose_a = skeleton.rest_pose.clone();
    let pose_b = skeleton.rest_pose.clone();
    let mut state = pollster::block_on(State::new(window));
    let diffuse_texture =
        pollster::block_on(load_texture("Woman.png", &state.device, &state.queue))
            .expect("Failed to read diffuse texture");
    let diffuse_texture = Arc::new(RwLock::new(diffuse_texture));
    let instances1 = Arc::new(RwLock::new(vec![Instance {
        position: Vector3 {
            x: 2.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Quaternion::default(),
    }]));
    let blend_between_clips = pollster::block_on(BlenderPlayer::new_blend_between_clips(
        vertices.clone(),
        original_positions.clone(),
        original_normals.clone(),
        indices.clone(),
        "Woman.gltf",
        &state.device,
        &state.config,
        &state.camera_persp_buffer,
        material.clone(),
        diffuse_texture.clone(),
        skeleton.clone(),
        instances1,
        current_pose.clone(),
        clip_a,
        clip_b,
        time_a,
        time_b,
        pose_a,
        pose_b,
    ))
    .unwrap();

    let clip_index = animation_clips
        .iter()
        .position(|c| c.name == "Walking")
        .unwrap()
        .to_owned();
    dbg!(animation_clips
        .iter()
        .map(|a| a.name.clone())
        .collect::<Vec<String>>());
    let add_pose = skeleton.rest_pose.clone();
    let additive_base = Pose::make_additive(&skeleton, &animation_clips[additive_index]);
    let instances2 = Arc::new(RwLock::new(vec![Instance {
        position: Vector3 {
            x: -2.0,
            y: 0.0,
            z: 0.0,
        },
        rotation: Quaternion::default(),
    }]));
    let layered_animation = pollster::block_on(BlenderPlayer::new_layered_animation(
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
        skeleton,
        instances2,
        current_pose,
        add_pose,
        additive_base,
        animation_clips,
        clip_index,
        additive_index,
    ))
    .unwrap();

    state.add_renderable(Renderable::BlenderPlayer(blend_between_clips));
    state.add_renderable(Renderable::BlenderPlayer(layered_animation));
    run(event_loop, state);
}
