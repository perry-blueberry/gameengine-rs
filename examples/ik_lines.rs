use gameengine_rs::{
    animation::{fabrik_solver::FabrikSolver, frame::Frame, transform_track::TransformTrack},
    camera::CameraPerspective,
    math::{glam_transform::Transform, vector3::Vector3},
    rendering::{
        line::LineRender,
        point::PointRender,
        render_players::ik_player::IkPlayer,
        renderable::{Renderable, SimpleVertex},
        state::State,
    },
};

use glam::{Quat, Vec3};
use winit::{event_loop::EventLoop, window::WindowBuilder};

use gameengine_rs::run;

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let mut state = pollster::block_on(State::new(window));
    let pitch = f32::to_radians(45.0);
    let yaw = f32::to_radians(60.0);
    let distance = 7.0;
    let eye = Vector3 {
        x: distance * yaw.cos() * pitch.sin(),
        y: distance * pitch.cos(),
        z: distance * yaw.sin() * pitch.sin(),
    };
    state.camera_persp = CameraPerspective {
        eye,
        target: [0.0, 0.0, 0.0].into(),
        up: Vector3::up(),
        aspect: state.config.width as f32 / state.config.height as f32,
        fovy: 60.0,
        znear: 0.01,
        zfar: 1000.0,
    };
    let mut solver = FabrikSolver::new();
    solver.resize(6);
    solver.set_local_transform(
        0,
        Transform::new(
            Vec3::ZERO,
            Quat::from_axis_angle(Vec3::new(1.0, 0.0, 0.0), f32::to_radians(90.0)),
            Vec3::new(1.0, 1.0, 1.0),
        ),
    );
    solver.set_local_transform(
        1,
        Transform::new(
            Vec3::new(0.0, 0.0, 1.0),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        ),
    );
    solver.set_local_transform(
        2,
        Transform::new(
            Vec3::new(0.0, 0.0, 1.5),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        ),
    );
    solver.set_local_transform(
        3,
        Transform::new(
            Vec3::new(0.0, 0.0, 0.5),
            Quat::from_axis_angle(Vec3::new(0.0, 1.0, 0.0), f32::to_radians(90.0)),
            Vec3::new(1.0, 1.0, 1.0),
        ),
    );
    solver.set_local_transform(
        4,
        Transform::new(
            Vec3::new(0.0, 0.0, 0.5),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        ),
    );
    solver.set_local_transform(
        5,
        Transform::new(
            Vec3::new(0.0, 0.0, 0.5),
            Quat::default(),
            Vec3::new(1.0, 1.0, 1.0),
        ),
    );

    let mut target_path = TransformTrack::new(0);
    let path = &mut target_path.position;
    path.frames
        .push(Frame::new_simple(0.0, Vec3::new(1.0, -2.0, 0.0) * 0.5));
    path.frames
        .push(Frame::new_simple(1.0, Vec3::new(1.0, 2.0, 0.0) * 0.5));
    path.frames
        .push(Frame::new_simple(2.0, Vec3::new(1.0, 4.0, 0.0) * 0.5));
    path.frames
        .push(Frame::new_simple(3.0, Vec3::new(3.0, 4.0, 0.0) * 0.5));
    path.frames
        .push(Frame::new_simple(4.0, Vec3::new(5.0, 4.0, 0.0) * 0.5));
    path.frames
        .push(Frame::new_simple(5.0, Vec3::new(5.0, 4.0, 2.0) * 0.5));
    path.frames
        .push(Frame::new_simple(6.0, Vec3::new(5.0, 4.0, 4.0) * 0.5));
    path.frames
        .push(Frame::new_simple(7.0, Vec3::new(3.0, 4.0, 4.0) * 0.5));
    path.frames
        .push(Frame::new_simple(8.0, Vec3::new(3.0, 2.0, 4.0) * 0.5));
    path.frames
        .push(Frame::new_simple(9.0, Vec3::new(3.0, 2.0, 2.0) * 0.5));
    path.frames
        .push(Frame::new_simple(10.0, Vec3::new(1.0, 2.0, 2.0) * 0.5));
    path.frames
        .push(Frame::new_simple(11.0, Vec3::new(1.0, 0.0, 2.0) * 0.5));
    path.frames
        .push(Frame::new_simple(12.0, Vec3::new(1.0, -2.0, 2.0) * 0.5));
    path.frames
        .push(Frame::new_simple(13.0, Vec3::new(1.0, -2.0, 0.0) * 0.5));

    let line_render = LineRender::new(
        vec![
            SimpleVertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 0.0, 0.0],
            };
            10
        ],
        &state.device,
        &state.config,
        &state.camera_persp_buffer,
        None,
    );
    let point_render = PointRender::new(
        &vec![
            SimpleVertex {
                position: [0.0, 0.0, 0.0],
                color: [1.0, 0.0, 0.0],
            };
            6
        ],
        &state.device,
        &state.config,
        &state.camera_persp_buffer,
    );
    let ik_player = IkPlayer::new(solver, target_path, point_render, line_render);
    state.add_ui_renderable(Renderable::IkPlayer(ik_player));
    run(event_loop, state);
}
