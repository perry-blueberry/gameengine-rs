use math::glam_transform::Transform;

use super::track::{QuatTrack, Vector3Track};

#[derive(Clone)]
pub struct TransformTrack {
    pub id: u32,
    pub position: Vector3Track,
    pub rotation: QuatTrack,
    pub scale: Vector3Track,
}

impl TransformTrack {
    pub fn new(id: u32) -> Self {
        Self {
            id,
            position: Vector3Track::new(),
            rotation: QuatTrack::new(),
            scale: Vector3Track::new(),
        }
    }

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

    pub fn sample(&self, ref_tf: Transform, t: f32, looping: bool) -> Transform {
        let mut result = ref_tf;
        if self.position.len() > 1 {
            result.translation = self.position.sample(t, looping);
        }
        if self.rotation.len() > 1 {
            result.rotation = self.rotation.sample(t, looping);
        }
        if self.scale.len() > 1 {
            result.scale = self.scale.sample(t, looping);
        }
        result
    }
}
