use std::sync::{Arc, RwLock};

use gameengine_rs::{
    animation::{
        clip::Clip,
        gltf_loader::{load_animation_clips, load_meshes, load_skeleton},
    },
    rendering::{renderable::Renderable, skeletal_model::SkeletalModel, state::State},
    resources::load_texture,
    run,
};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(window));
    let (document, buffers, _images) = gltf::import("res/Woman.gltf").expect("Failed to open gltf");
    let (vertices, original_positions, original_normals, indices, material) =
        load_meshes(&document, &buffers);
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
        current_clip,
        skeleton,
    ))
    .unwrap();
    state.add_renderable(Renderable::SkeletalModel(model));
    run(event_loop, state);
}
