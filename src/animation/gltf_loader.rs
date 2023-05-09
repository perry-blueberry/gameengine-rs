use std::collections::HashMap;
use std::ops::Neg;

use cgmath::{
    InnerSpace, Matrix3, Matrix4, Quaternion, Rotation, SquareMatrix, Transform, Vector3,
};
use gltf::animation::util::ReadOutputs;
use gltf::{animation::Channel, buffer::Data, Document};
use gltf::{Material, Node, Skin};

use crate::rendering::skeletal_model::SkeletalVertex;

use super::pose;
use super::skeleton::Skeleton;
use super::{
    array_type::ArrayType,
    clip::Clip,
    frame::Frame,
    interpolation::Interpolation,
    pose::Pose,
    track::{DefaultConstructible, Track},
};

pub fn load_skeleton(data: &Document, buffer_data: &Vec<Data>) -> Skeleton {
    Skeleton::new(
        load_rest_pose(data),
        load_bind_pose(data, buffer_data),
        load_joint_names(data),
    )
}

pub fn load_rest_pose(data: &Document) -> Pose {
    let mut result = Pose::new();
    let children_to_parent = children_to_parent(data);
    for (i, node) in data.nodes().enumerate() {
        let transform = match node.transform() {
            gltf::scene::Transform::Matrix { matrix: _ } => panic!("Matrix not implemented"),
            gltf::scene::Transform::Decomposed {
                translation,
                rotation,
                scale,
            } => pose::Transform {
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

pub fn load_bind_pose(data: &Document, buffer_data: &Vec<Data>) -> Pose {
    let rest_pose = load_rest_pose(data);
    let num_bones = rest_pose.len();
    let mut world_bind_pose = Vec::with_capacity(num_bones);
    for i in 0..num_bones {
        world_bind_pose.push(rest_pose.global_transform(i));
    }
    for skin in data.skins() {
        let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
        let inverse_bind_accessor: Vec<[[f32; 4]; 4]> = reader
            .read_inverse_bind_matrices()
            .expect("Failed to read inverse bind matrices")
            .collect();
        for (i, joint) in skin.joints().enumerate() {
            // It's already an inverse so the inverse exists
            let bind_matrix = Matrix4::from(inverse_bind_accessor[i])
                .inverse_transform()
                .unwrap();
            world_bind_pose[joint.index()] = matrix_to_decomposed(bind_matrix);
        }
    }
    let mut bind_pose = rest_pose;
    for (i, current) in world_bind_pose.iter().enumerate() {
        let mut current = current.to_owned();
        if let Some(parent) = bind_pose.parent(i) {
            let parent_transform = world_bind_pose[parent];
            current = parent_transform
                .inverse_transform()
                .unwrap()
                .concat(&current);
        }
        bind_pose.set_local_transform(i, current);
    }
    bind_pose
}

fn children_to_parent(data: &Document) -> HashMap<usize, usize> {
    let mut result = HashMap::new();
    for parent in data.nodes() {
        for child in parent.children() {
            result.insert(child.index(), parent.index());
        }
    }
    result
}

fn load_joint_names(data: &Document) -> Vec<String> {
    let mut res = vec![];
    for node in data.nodes() {
        match node.name() {
            Some(n) => res.push(n.into()),
            None => res.push("EMPTY NODE".into()),
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

pub fn load_meshes<'a>(
    data: &'a Document,
    buffer_data: &Vec<Data>,
) -> (
    Vec<SkeletalVertex>,
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<u32>,
    Material<'a>,
) {
    let skin = &data.skins().collect::<Vec<Skin>>()[0];
    let skin_joints: Vec<Node> = skin.joints().collect();
    /* dbg!(skin_joints.iter().map(|j| j.index()).collect()); */
    let mut vertices = vec![];
    let reader = skin.reader(|buffer| Some(&buffer_data[buffer.index()]));
    let inv_bind_matrices: Vec<Matrix4<f32>> = reader
        .read_inverse_bind_matrices()
        .unwrap()
        .map(|m| m.into())
        .collect();
    for mesh in data.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffer_data[buffer.index()]));
            let positions: Vec<[f32; 3]> = reader
                .read_positions()
                .expect("Failed to read positions")
                .collect();
            let tex_coords: Vec<[f32; 2]> = reader
                .read_tex_coords(0)
                .expect("Failed to read tex_coords")
                .into_f32()
                .collect();
            let normals: Vec<[f32; 3]> = reader
                .read_normals()
                .expect("Failed to read normals")
                .collect();
            let weights: Vec<[f32; 4]> = reader
                .read_weights(0)
                .expect("Failed to read weights")
                .into_f32()
                .collect();
            let joints: Vec<[u16; 4]> = reader
                .read_joints(0)
                .expect("Failed to read joints")
                .into_u16()
                .collect();
            let indices: Vec<u32> = reader
                .read_indices()
                .expect("Failed to read indices")
                .into_u32()
                .collect();
            assert_eq!(positions.len(), tex_coords.len());
            assert_eq!(positions.len(), normals.len());
            assert_eq!(positions.len(), weights.len());
            assert_eq!(positions.len(), joints.len());

            for (i, pos) in positions.iter().enumerate() {
                let tex_coords = tex_coords[i];
                let normal = normals[i];
                let weights = weights[i];
                let joints = joints[i];
                let joints = [
                    skin_joints[joints[0] as usize].index(),
                    skin_joints[joints[1] as usize].index(),
                    skin_joints[joints[2] as usize].index(),
                    skin_joints[joints[3] as usize].index(),
                ];
                vertices.push(SkeletalVertex {
                    position: *pos,
                    tex_coords,
                    normal,
                    weights,
                    joints: [
                        joints[0] as u16,
                        joints[1] as u16,
                        joints[2] as u16,
                        joints[3] as u16,
                    ],
                });
            }
            return (vertices, positions, normals, indices, primitive.material());
        }
    }
    panic!("GLTF didn't have any primitives");
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
                //TODO: Decide how last value should be handled
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
        //TODO: Decide how last value should be handled
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
    frames
}

fn matrix_to_quaternion(matrix: &Matrix4<f32>) -> Quaternion<f32> {
    let up = matrix.y.truncate().normalize();
    let forward = matrix.z.truncate().normalize();
    let right = up.cross(forward);
    let up = forward.cross(right);
    Matrix3::look_to_rh(forward, up).into()
    /* Quaternion::look_at(-forward, up) */
}

fn matrix_to_decomposed(m: Matrix4<f32>) -> pose::Transform {
    let disp = Vector3 {
        x: m.w.x,
        y: m.w.y,
        z: m.w.z,
    };
    let rot = {
        let rot = matrix_to_quaternion(&m);
        // TODO: Figure out why z has to be inverted
        Quaternion::new(rot.s, rot.v.x, rot.v.y, -rot.v.z)
    };

    let rot_scale_matrix = Matrix4::new(
        m.x.x, m.x.y, m.x.z, 0.0, m.y.x, m.y.y, m.y.z, 0.0, m.z.x, m.z.y, m.z.z, 0.0, 0.0, 0.0,
        0.0, 1.0,
    );
    let inv_rot_matrix: Matrix4<f32> = rot.invert().into();
    let scale_skew_matrix = rot_scale_matrix.concat(&inv_rot_matrix);
    let scale = scale_skew_matrix.diagonal().truncate();
    /* let scale = vec![
        matrix.x.truncate().magnitude(),
        matrix.y.truncate().magnitude(),
        matrix.z.truncate().magnitude(),
    ];
    let rotation_matrix = Matrix4::from_nonuniform_scale(scale[0], scale[1], scale[2])
        .invert()
        .unwrap()
        * matrix;

    /* let rot = Quaternion::from([
        rotation_matrix.x,
        rotation_matrix.y,
        rotation_matrix.x,
        rotation_matrix.w,
    ]); */
    let trace = rotation_matrix.trace() - 1.0;
    let s = (trace / 4.0).sqrt();

    let w = s;
    let x = (rotation_matrix[2][1] - rotation_matrix[1][2]) / (4.0 * s);
    let y = (rotation_matrix[0][2] - rotation_matrix[2][0]) / (4.0 * s);
    let z = (rotation_matrix[1][0] - rotation_matrix[0][1]) / (4.0 * s);

    let rot = Quaternion::new(w, x, y, z);
    let disp = matrix.w.truncate(); */
    // TODO: Use full scale
    pose::Transform {
        scale: scale[0],
        rot,
        disp,
    }
}
