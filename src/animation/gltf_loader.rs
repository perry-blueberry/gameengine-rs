use std::collections::HashMap;

use cgmath::One;
use gltf::{json::Accessor, Document, Gltf};

use super::pose::{Pose, Transform};

pub struct GltfLoader {}

pub fn load_rest_pose(data: &Document /* &Gltf */) -> Pose {
    let mut result = Pose::new();
    let children_to_parent = children_to_parent(data);
    for (i, node) in data.nodes().enumerate() {
        let transform = match node.transform() {
            gltf::scene::Transform::Matrix { matrix } => todo!(),
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
        result.set_local_transform(i, transform);
        if let Some(parent) = children_to_parent.get(&i) {
            result.set_parent(i, *parent);
        } else {
            result.unset_parent(i);
        }
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

fn scalar_values(out_scalars: &mut Vec<f32>, in_accessor: &Accessor) {}

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
