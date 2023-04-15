use super::track::{QuatTrack, Vector3Track};

use cgmath::{Decomposed, Quaternion, Vector3};

pub struct TransformTrack {
    pub id: u32,
    position: Vector3Track,
    rotation: QuatTrack,
    scale: Vector3Track,
}

impl TransformTrack {
    pub fn is_valid(&self) -> bool {
        self.position.len() > 0 || self.rotation.len() > 0 || self.scale.len() > 0
    }

    pub fn start_time(&self) -> Option<f32> {
        [
            self.position.start_time(),
            self.rotation.start_time(),
            self.scale.start_time(),
        ]
        .into_iter()
        .filter_map(|s| s)
        .reduce(f32::min)
    }

    pub fn end_time(&self) -> Option<f32> {
        [
            self.position.end_time(),
            self.rotation.end_time(),
            self.scale.end_time(),
        ]
        .into_iter()
        .filter_map(|s| s)
        .reduce(f32::min)
    }

    pub fn sample(
        &self,
        ref_tf: Decomposed<Vector3<f32>, Quaternion<f32>>,
        t: f32,
        looping: bool,
    ) -> Decomposed<Vector3<f32>, Quaternion<f32>> {
        let mut result = ref_tf;
        if self.position.len() > 1 {
            result.disp = self.position.sample(t, looping);
        }
        if self.rotation.len() > 1 {
            result.rot = self.rotation.sample(t, looping);
        }
        if self.scale.len() > 1 {
            result.scale = self.scale.sample(t, looping)[0];
        }
        result
    }
}
