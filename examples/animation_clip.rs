use animation::clip::Clip;
use gameengine_rs::run;
use gameengine_rs::state::State;
use rendering::{
    gltf_loader::{load_animation_clips, load_rest_pose},
    line::LineRender,
    render_players::animation_clip_player::{from_pose, AnimationClipPlayer},
    renderable::Renderable,
    texture,
};
use wgpu::{CompareFunction, DepthBiasState, DepthStencilState, StencilState};
use winit::{event_loop::EventLoop, window::WindowBuilder};

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let (document, buffers, _images) = gltf::import("res/Woman.gltf").expect("Failed to open gltf");

    let rest_pose = load_rest_pose(&document);
    let rest_pose_lines = from_pose(&rest_pose, [1.0, 0.0, 0.0]);

    let animation_clips: Vec<Clip> = load_animation_clips(&document, &buffers);
    let current_clip = animation_clips
        .iter()
        .find(|c| c.name == "Walking")
        .unwrap()
        .to_owned();
    let current_pose = rest_pose;
    let mut state = pollster::block_on(State::new(window));
    let animation_clip_player = AnimationClipPlayer::new(
        current_clip,
        &state.device,
        &state.config,
        &state.camera_persp_buffer,
        current_pose,
    );
    let line_render = LineRender::new(
        rest_pose_lines,
        &state.device,
        &state.config,
        &state.camera_persp_buffer,
        Some(DepthStencilState {
            format: texture::Texture::DEPTH_FORMAT,
            depth_write_enabled: true,
            depth_compare: CompareFunction::Less,
            stencil: StencilState::default(),
            bias: DepthBiasState::default(),
        }),
    );

    state.add_renderable(Renderable::Line(line_render));
    state.add_renderable(Renderable::AnimationClipPlayer(animation_clip_player));
    run(event_loop, state);
}
