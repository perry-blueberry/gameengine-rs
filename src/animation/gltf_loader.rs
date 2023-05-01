use std::collections::HashMap;

use cgmath::{Quaternion, Vector3};
use gltf::{accessor::Iter, animation::util::ReadOutputs};
use gltf::{animation::Channel, buffer::Data, json::Accessor, Document, Gltf};

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
    /*     let num_clips = data.animations().len();
    let num_nodes = data.nodes().len(); */
    for (i, animation) in data.animations().enumerate() {
        let name = animation.name();
        for channel in animation.channels() {
            let node_id = channel.target().node().index() as u32;
            if results.get(i).is_none() {
                results.push(Clip::new(name));
            }
            let (frames, interp) = frames_from_channel(&channel, buffer_data);
            /* results[i].add_track(TransformTrack::new(id)) */
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

/* struct DataUri<'a> {
    mime_type: &'a str,
    base64: bool,
    data: &'a str,
}

fn split_once(input: &str, delimiter: char) -> Option<(&str, &str)> {
    let mut iter = input.splitn(2, delimiter);
    Some((iter.next()?, iter.next()?))
}

impl<'a> DataUri<'a> {
    fn parse(uri: &'a str) -> Result<DataUri<'a>, ()> {
        let uri = uri.strip_prefix("data:").ok_or(())?;
        let (mime_type, data) = split_once(uri, ',').ok_or(())?;

        let (mime_type, base64) = match mime_type.strip_suffix(";base64") {
            Some(mime_type) => (mime_type, true),
            None => (mime_type, false),
        };

        Ok(DataUri {
            mime_type,
            base64,
            data,
        })
    }

    fn decode(&self) -> Result<Vec<u8>, base64::DecodeError> {
        if self.base64 {
            base64::decode(self.data)
        } else {
            Ok(self.data.as_bytes().to_owned())
        }
    }
} */

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
    let mut timeline_floats: Vec<f32> = reader
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
            let mut fs = fs.into_f32();
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
            /* for i in (0..timeline_floats.len()).step_by(3) {
                let time = timeline_floats
                    .nth(i)
                    .expect(&format!("Failed to read input {}", i));
                let value =
                    Q::from_slice(&fs.nth(i + 1).expect(&format!("Failed to read value {}", i)));
                let (in_tangent, out_tangent) = if is_sampler_cubic {
                    (
                        Q::from_slice(
                            &fs.nth(i)
                                .expect(&format!("Failed to read in_tangent {}", i)),
                        ),
                        Q::from_slice(
                            &fs.nth(i + 2)
                                .expect(&format!("Failed to read out_tangent {}", i)),
                        ),
                    )
                } else {
                    (Q::default(), Q::default())
                };
                frames.push(Frame::new(time, in_tangent, out_tangent, value));
            } */
            (TransformComponentVec::Rotation(frames), interpolation)
        }
        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
    }
}

fn frames_from_channel_vec3(
    mut timeline_floats: Vec<f32>,
    mut fs: Vec<[f32; 3]>,
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

/* fn track_from_channel<T>(track: &mut Track<T>, channel: &Channel)
where
    T: Neighborhood
        + AdjustHermiteResult
        + Copy
        + Mul<f32, Output = T>
        + Add<Output = T>
        + DefaultConstructible
        + ArrayType
        + Interpolate,
{
    let sampler: gltf::animation::Sampler = channel.sampler();
    let interpolation = match sampler.interpolation() {
        gltf::animation::Interpolation::Linear => Interpolation::Linear,
        gltf::animation::Interpolation::Step => Interpolation::Constant,
        gltf::animation::Interpolation::CubicSpline => Interpolation::Cubic,
    };
    track.set_interpolation(interpolation);
    let input_accessor = sampler.input();
    let input_buffer_view = input_accessor.view().unwrap();
    let input_raw_buffer_data: gltf::buffer::Source = input_buffer_view.buffer().source();
    let buffer_data: Vec<Data> = vec![];
    /* let input_data = input_accessor.read; */
    /* json::deserialize::from_value::<f32>(input_accessor.min().unwrap()); */
    let reader = channel.reader(|buffer| Some(&buffer_data[buffer.index()]));
    let timeline_floats = /* vec![ */
        reader.read_inputs().expect("Failed to read inputs") /* .nth(0)
                                                              .expect("Failed to get 0th") */
    /* ] */;
    let is_sampler_cubic = interpolation == Interpolation::Cubic;
    match reader.read_outputs().expect("Failed to read outputs") {
        gltf::animation::util::ReadOutputs::Translations(mut fs) => {
            /* V3::from_slice( */
            /* let fs = fs.nth(0).unwrap();
            vec![fs[0], fs[1], fs[2]] */
            /* fs.map(|f| f).collect::<Vec<f32>>() */
            /* Vec::from_iter(fs.into_iter()) */
            /*  & fs.nth(0).unwrap().to_vec().to_owned() */
            /* ) */
            for i in (0..timeline_floats.len()).step_by(3) {
                let frame = track.frame(i);
                frame.time = timeline_floats
                    .nth(i)
                    .expect(&format!("Failed to read input {}", i));
                frame.in_tangent = V3::from_slice(
                    value_floats
                        .nth(i)
                        .expect(&format!("Failed to read input {}", i)),
                );
            }
            for translation in fs {}
        }
        gltf::animation::util::ReadOutputs::Rotations(fs) =>
        /* Q::from_slice( */
        {
            let fs = fs.into_f32().nth(0).unwrap();
            vec![fs[0], fs[1], fs[2], fs[3]]
            /* vec![fs.nth(0).unwrap(), fs.nth(1).unwrap(), fs.nth(2).unwrap()] */
            /* Vec::from_iter(fs.into_f32().into_iter()) */
            /* &fs.into_f32().nth(0).unwrap().to_vec().to_owned() */
        } /* ) */
        gltf::animation::util::ReadOutputs::Scales(mut fs) => {
            let fs = fs.nth(0).unwrap();
            vec![fs[0], fs[1], fs[2]]
        }
        gltf::animation::util::ReadOutputs::MorphTargetWeights(_) => todo!(),
    };
    let num_frames = sampler.input().count();
}
 */
/* async fn load_buffers(source: Source<'static>) -> Vec<Vec<u8>> {
    const VALID_MIME_TYPES: &[&str] = &["application/octet-stream", "application/gltf-buffer"];

    let mut buffer_data = Vec::new();
    match source {
        gltf::buffer::Source::Uri(uri) => {
            let uri = percent_encoding::percent_decode_str(uri)
                .decode_utf8()
                .unwrap();
            let uri = uri.as_ref();
            let buffer_bytes = match DataUri::parse(uri) {
                Ok(data_uri) if VALID_MIME_TYPES.contains(&data_uri.mime_type) => data_uri.decode(),
                Ok(_) => panic!("Buffer format unsupported"),
                Err(()) => {
                    // TODO: Remove this and add dep
                    let buffer_path = asset_path.parent().unwrap().join(uri);
                    load_context.read_asset_bytes(buffer_path).await
                }
            };
            buffer_data.push(buffer_bytes);
        }
        gltf::buffer::Source::Bin => {
            if let Some(blob) = gltf.blob.as_deref() {
                buffer_data.push(blob.into());
            } else {
                panic!("Missing blob");
            }
        }
    }

    Ok(buffer_data)
} */

/* fn scalar_values(out_scalars: &mut Vec<f32>, in_accessor: &Accessor) {} */

/* fn get_local_transform(node: &Node) -> Transform {
    let mut result = Transform::one();
    if let Some(matrix) = node.matrix {
        //TODO: Convert matrix to Transform
    } else {
        if let Some(t) = node.translation {
            result.disp = t.into();
        } else {
            println!("No translation found for {:?}", node.name);
        }
        if let Some(r) = node.rotation {
            result.rot = r.0.into();
        } else {
            println!("No rotation found for {:?}", node.name);
        }
        /* if let Some(s) = node.scale {
            result.scale = s.into();
        } else {
            println!("No scale found for {:?}", node.name);
        } */
    }
    result
} */

/* fn decompose(matrix: [f32; 16]) -> Transform {
    let position = Vector3::new(matrix[12], matrix[13], matrix[14]);
} */
