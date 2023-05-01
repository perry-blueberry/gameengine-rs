use std::collections::HashMap;

use cgmath::{Quaternion, Vector3};
use gltf::animation::util::ReadOutputs;
use gltf::{animation::Channel, buffer::Data, Document, Gltf};

use super::{
    array_type::ArrayType,
    clip::Clip,
    frame::Frame,
    interpolation::Interpolation,
    pose::{Pose, Transform},
    track::{DefaultConstructible, Track},
};

pub struct GltfLoader {}

pub fn load_rest_pose(data: &Document /* &Gltf */) -> Pose {
    let mut result = Pose::new();
    let children_to_parent = children_to_parent(data);
    for (i, node) in data.nodes().enumerate() {
        let transform = match node.transform() {
            gltf::scene::Transform::Matrix { matrix: _ } => panic!("Matrix not implemented"),
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => Transform {
                scale: scale[0],
                rot: rotation.into(),
                disp: translation.into(),
            },
        };
        result.add_local_transform(transform);
        result.add_parent(children_to_parent.get(&i).copied());
    }
    result
}

fn children_to_parent(data: &Document /* &Gltf */) -> HashMap<usize, usize> {
    let mut result = HashMap::new();
    for parent in data.nodes() {
        for child in parent.children() {
            result.insert(child.index(), parent.index());
        }
    }
    result
}

fn load_joint_names(data: &Gltf) -> Vec<&str> {
    let mut res = vec![];
    for node in data.nodes() {
        match node.name() {
            Some(n) => res.push(n),
            None => res.push("EMPTY NODE"),
        }
    }
    res
}

pub fn load_animation_clips(data: &Document, buffer_data: &Vec<Data>) -> Vec<Clip> {
    let mut results: Vec<Clip> = vec![];
    for (i, animation) in data.animations().enumerate() {
        let name = animation.name();
        for channel in animation.channels() {
            let node_id = channel.target().node().index() as u32;
            if results.get(i).is_none() {
                results.push(Clip::new(name));
            }
            let (frames, interp) = frames_from_channel(&channel, buffer_data);
            let transform_track = results[i].transform_track(node_id);

            match frames {
                TransformComponentVec::Translation(t) => {
                    transform_track.position = Track::new_with_args(interp, t);
                }
                TransformComponentVec::Rotation(r) => {
                    transform_track.rotation = Track::new_with_args(interp, r);
                }
                TransformComponentVec::Scale(s) => {
                    transform_track.scale = Track::new_with_args(interp, s);
                }
            };

            results[i].recalculate_duration();
        }
    }
    results
}

type V3 = Vector3<f32>;
type Q = Quaternion<f32>;

pub enum TransformComponentVec {
    Translation(Vec<Frame<Vector3<f32>>>),
    Rotation(Vec<Frame<Quaternion<f32>>>),
    Scale(Vec<Frame<Vector3<f32>>>),
}

fn frames_from_channel(
    channel: &Channel,
    buffer_data: &Vec<Data>,
) -> (TransformComponentVec, Interpolation) {
    let interpolation = match channel.sampler().interpolation() {
        gltf::animation::Interpolation::Linear => Interpolation::Linear,
        gltf::animation::Interpolation::Step => Interpolation::Constant,
        gltf::animation::Interpolation::CubicSpline => Interpolation::Cubic,
    };
    let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
    let timeline_floats: Vec<f32> = reader
        .read_inputs()
        .expect("Failed to read inputs")
        .collect();
    let is_sampler_cubic = interpolation == Interpolation::Cubic;
    match reader.read_outputs().expect("Failed to read outputs") {
        ReadOutputs::Translations(fs) => {
            let fs: Vec<[f32; 3]> = fs.collect();
            let frames = frames_from_channel_vec3(timeline_floats, fs, is_sampler_cubic);
            (TransformComponentVec::Translation(frames), interpolation)
        }
        ReadOutputs::Scales(fs) => {
            let fs: Vec<[f32; 3]> = fs.collect();
            let frames = frames_from_channel_vec3(timeline_floats, fs, is_sampler_cubic);
            (TransformComponentVec::Scale(frames), interpolation)
        }
        ReadOutputs::Rotations(fs) => {
            let fs = fs.into_f32();
            let fs: Vec<[f32; 4]> = fs.collect();
            let mut frames = vec![];
            assert_eq!(fs.len(), timeline_floats.len());
            for i in 0..timeline_floats.len() {
                let time = timeline_floats[i];
                let value = if let Some(value) = fs.get(i + 1) {
                    Q::from_slice(value)
                } else {
                    Q::from_slice(&fs[0])
                };
                let (in_tangent, out_tangent) = if is_sampler_cubic {
                    (Q::from_slice(&fs[i]), Q::from_slice(&fs[i + 2]))
                } else {
                    (Q::default(), Q::default())
                };
                frames.push(Frame::new(time, in_tangent, out_tangent, value));
            }
            (TransformComponentVec::Rotation(frames), interpolation)
        }
        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
    }
}

fn frames_from_channel_vec3(
    timeline_floats: Vec<f32>,
    fs: Vec<[f32; 3]>,
    is_sampler_cubic: bool,
) -> Vec<Frame<Vector3<f32>>> {
    let mut frames = vec![];
    for i in 0..timeline_floats.len() {
        let time = timeline_floats[i];
        let value = if let Some(value) = fs.get(i + 1) {
            V3::from_slice(value)
        } else {
            V3::from_slice(&fs[0])
        };
        let (in_tangent, out_tangent) = if is_sampler_cubic {
            (V3::from_slice(&fs[i]), V3::from_slice(&fs[i + 2]))
        } else {
            (V3::default(), V3::default())
        };
        frames.push(Frame::new(time, in_tangent, out_tangent, value));
    }

    dbg!(frames.len());
    frames
}
