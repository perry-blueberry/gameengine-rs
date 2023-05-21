use gameengine_rs::{
    animation::{
        clip::Clip,
        gltf_loader::{load_animation_clips, load_meshes, load_skeleton},
    },
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
        load_meshes(&document, &buffers);
    let skeleton = load_skeleton(&document, &buffers);

    let animation_clips: Vec<Clip> = load_animation_clips(&document, &buffers);
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
    let blender_player = pollster::block_on(BlenderPlayer::new(
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
        current_pose,
        clip_a,
        clip_b,
        time_a,
        time_b,
        pose_a,
        pose_b,
    ))
    .unwrap();

    state.add_renderable(Renderable::BlenderPlayer(blender_player));
    run(event_loop, state);
}
