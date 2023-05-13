use gameengine_rs::{
    animation::{frame::ScalarFrame, interpolation::Interpolation, track::ScalarTrack},
    math::vector3::Vector3,
    rendering::{
        line::LineRender,
        point::PointRender,
        renderable::{Renderable, SimpleVertex},
        state::State,
    },
};

use winit::{event_loop::EventLoop, window::WindowBuilder};

use gameengine_rs::run;

pub fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let sample = Sample::new();
    let lines = [
        sample.ref_lines,
        sample.scalar_track_lines,
        sample.handle_lines,
    ]
    .into_iter()
    .flatten()
    .collect();
    let mut state = pollster::block_on(State::new(window));
    let line_render = LineRender::new(
        lines,
        &state.device,
        &state.config,
        &state.camera_ortho_buffer,
        None,
    );
    state.add_ui_renderable(Renderable::Line(line_render));
    let point_render = PointRender::new(
        &sample.handle_points,
        &state.device,
        &state.config,
        &state.camera_ortho_buffer,
    );
    state.add_ui_renderable(Renderable::Point(point_render));
    run(event_loop, state);
}

struct Sample {
    ref_lines: Vec<SimpleVertex>,
    scalar_track_lines: Vec<SimpleVertex>,
    handle_lines: Vec<SimpleVertex>,
    handle_points: Vec<SimpleVertex>,
}

impl Sample {
    fn new() -> Self {
        let mut scalar_tracks: Vec<ScalarTrack> = vec![];
        let mut scalar_tracks_looping: Vec<bool> = vec![];
        let mut ref_lines: Vec<SimpleVertex> = vec![];
        let mut scalar_track_lines: Vec<SimpleVertex> = vec![];
        let mut handle_lines: Vec<SimpleVertex> = vec![];
        let mut handle_points: Vec<SimpleVertex> = vec![];

        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Linear,
            vec![
                ScalarFrame::new_simple(0.0, 0.0),
                ScalarFrame::new_simple(1.0, 1.0),
            ],
        ));
        scalar_tracks_looping.push(false);
        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Linear,
            vec![
                ScalarFrame::new_simple(0.0, 0.0),
                ScalarFrame::new_simple(0.5, 1.0),
            ],
        ));
        scalar_tracks_looping.push(false);
        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Linear,
            vec![
                ScalarFrame::new_simple(0.25, 0.0),
                ScalarFrame::new_simple(0.5, 1.0),
                ScalarFrame::new_simple(0.75, 0.0),
            ],
        ));
        scalar_tracks_looping.push(true);
        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Linear,
            vec![
                ScalarFrame::new_simple(0.25, 0.0),
                ScalarFrame::new_simple(0.5, 1.0),
                ScalarFrame::new_simple(0.75, 0.0),
            ],
        ));
        scalar_tracks_looping.push(false);
        for _ in 0..2 {
            let step_frames = (0..11)
                .into_iter()
                .map(|i| {
                    let time = i as f32 / 10.0 * 0.5 + 0.25;
                    ScalarFrame::new_simple(time, if i % 2 == 0 { 0.0 } else { 1.0 })
                })
                .collect();
            let step_track = ScalarTrack::new_with_args(Interpolation::Constant, step_frames);
            scalar_tracks.push(step_track);
        }
        scalar_tracks_looping.push(true);
        scalar_tracks_looping.push(false);

        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Cubic,
            vec![
                ScalarFrame::new(0.25, 0.676221, 0.676221, 0.0),
                ScalarFrame::new(0.75, 4.043837, 4.043837, 1.0),
            ],
        ));
        scalar_tracks_looping.push(false);
        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Cubic,
            vec![
                ScalarFrame::new(0.25, 0.0, 0.0, 0.0),
                ScalarFrame::new(0.5, 0.0, 0.0, 1.0),
                ScalarFrame::new(0.75, 0.0, 0.0, 0.0),
            ],
        ));
        scalar_tracks_looping.push(true);
        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Cubic,
            vec![
                ScalarFrame::new(0.25, 0.0, 0.0, 0.0),
                ScalarFrame::new(0.3833333, -10.11282, -10.11282, 0.5499259),
                ScalarFrame::new(0.5, 25.82528, 25.82528, 1.0),
                ScalarFrame::new(0.6333333, 7.925411, 7.925411, 0.4500741),
                ScalarFrame::new(0.75, 0.0, 0.0, 0.0),
            ],
        ));
        scalar_tracks_looping.push(true);
        scalar_tracks.push(ScalarTrack::new_with_args(
            Interpolation::Cubic,
            vec![
                ScalarFrame::new(0.25, 0.0, 0.0, 0.0),
                ScalarFrame::new(0.3833333, 13.25147, -10.11282, 0.5499259),
                ScalarFrame::new(0.5, 10.2405, -5.545671, 1.0),
                ScalarFrame::new(0.6333333, 7.925411, -11.40672, 0.4500741),
                ScalarFrame::new(0.75, 0.0, 0.0, 0.0),
            ],
        ));
        scalar_tracks_looping.push(true);

        let height = 1.8;
        let left = 1.0;
        let right = 14.0;
        let x_range = right - left;

        for i in 0..10 {
            let y_pos = (i as f32 * 2.0) + (i as f32 * 0.2) + 0.1;

            ref_lines.push(SimpleVertex {
                position: [left, y_pos, 0.0],
                color: [1.0, 1.0, 1.0],
            });
            ref_lines.push(SimpleVertex {
                position: [left, y_pos + height, 0.0],
                color: [1.0, 1.0, 1.0],
            });
            ref_lines.push(SimpleVertex {
                position: [left, y_pos, 0.0],
                color: [1.0, 1.0, 1.0],
            });
            ref_lines.push(SimpleVertex {
                position: [right, y_pos, 0.0],
                color: [1.0, 1.0, 1.0],
            });
        }

        for i in 0..scalar_tracks.len() {
            let y_pos = ((9 - i) as f32 * 2.0) + ((9 - i) as f32 * 0.2) + 0.1;

            for j in 1..150 {
                let this_j_norm = (j - 1) as f32 / 149.0;
                let next_j_norm = j as f32 / 149.0;

                let this_x = left + this_j_norm * x_range;
                let next_x = left + next_j_norm * x_range;

                let this_y = {
                    let this_y = scalar_tracks[i].sample(this_j_norm, scalar_tracks_looping[i]);
                    y_pos + this_y * height
                };
                let next_y = {
                    let next_y = scalar_tracks[i].sample(next_j_norm, scalar_tracks_looping[i]);
                    y_pos + next_y * height
                };

                scalar_track_lines.push(SimpleVertex {
                    position: [this_x, this_y, 0.1],
                    color: [0.0, 1.0, 0.0],
                });
                scalar_track_lines.push(SimpleVertex {
                    position: [next_x, next_y, 0.1],
                    color: [0.0, 1.0, 0.0],
                });
            }

            let num_frames = scalar_tracks[i].len();
            for j in 0..num_frames {
                let this_time = scalar_tracks[i].frame(j).time;
                let this_y =
                    y_pos + scalar_tracks[i].sample(this_time, scalar_tracks_looping[i]) * height;
                let this_x = left + this_time * x_range;
                handle_points.push(SimpleVertex {
                    position: [this_x, this_y, 0.9],
                    color: [0.0, 0.0, 1.0],
                });

                if j > 0 {
                    let prev_y = y_pos
                        + scalar_tracks[i].sample(this_time - 0.0005, scalar_tracks_looping[i])
                            * height;
                    let prev_x = left + (this_time - 0.0005) * x_range;

                    let this_vec = Vector3 {
                        x: this_x,
                        y: this_y,
                        z: 0.6,
                    };
                    let prev_vec = Vector3 {
                        x: prev_x,
                        y: prev_y,
                        z: 0.6,
                    };
                    let handle_vec = this_vec + (prev_vec - this_vec).normalized() * 0.75;

                    handle_lines.push(SimpleVertex {
                        position: [this_vec.x, this_vec.y, this_vec.z],
                        color: [1.0, 0.0, 0.0],
                    });
                    handle_lines.push(SimpleVertex {
                        position: [handle_vec.x, handle_vec.y, handle_vec.z],
                        color: [1.0, 0.0, 0.0],
                    });
                }
                if j < num_frames - 1 && scalar_tracks[i].interpolation() != Interpolation::Constant
                {
                    let next_y = y_pos
                        + scalar_tracks[i].sample(this_time + 0.0005, scalar_tracks_looping[i])
                            * height;
                    let next_x = left + (this_time + 0.0005) * x_range;

                    let this_vec = Vector3 {
                        x: this_x,
                        y: this_y,
                        z: 0.6,
                    };
                    let next_vec = Vector3 {
                        x: next_x,
                        y: next_y,
                        z: 0.6,
                    };
                    let handle_vec = this_vec + (next_vec - this_vec).normalized() * 0.75;

                    handle_lines.push(SimpleVertex {
                        position: [this_vec.x, this_vec.y, this_vec.z],
                        color: [1.0, 0.0, 0.0],
                    });
                    handle_lines.push(SimpleVertex {
                        position: [handle_vec.x, handle_vec.y, handle_vec.z],
                        color: [1.0, 0.0, 0.0],
                    });
                }
            }
        }

        Self {
            ref_lines,
            scalar_track_lines,
            handle_lines,
            handle_points,
        }
    }
}
