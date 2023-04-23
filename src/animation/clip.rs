use cgmath::num_traits::clamp;

use super::{pose::Pose, track::loop_time, transform_track::TransformTrack};

pub struct Clip {
    tracks: Vec<TransformTrack>,
    pub name: &'static str,
    start_time: f32,
    end_time: f32,
    pub looping: bool,
}

impl Clip {
    pub fn new() -> Self {
        Self {
            tracks: vec![],
            name: "No name given",
            start_time: 0.0,
            end_time: 0.0,
            looping: true,
        }
    }

    pub fn sample(&self, out_pose: &mut Pose, time: f32) -> f32 {
        if self.duration() == 0.0 {
            return 0.0;
        }
        let time = self.adjust_time_to_fit_range(time);
        for track in &self.tracks {
            let joint = track.id as usize;
            let local = out_pose.local_transform(joint);
            let animated = track.sample(local, time, self.looping);
            out_pose.set_local_transform(joint, animated);
        }
        time
    }

    pub fn recalculate_duration(&mut self) {
        let start_time = 0.0;
        let end_time = 0.0;
        if let Some(s) = self
            .tracks
            .iter()
            .filter(|t| t.is_valid())
            .filter_map(|t| t.start_time())
            .reduce(f32::min)
        {
            self.start_time = s;
        }
        if let Some(e) = self
            .tracks
            .iter()
            .filter(|t| t.is_valid())
            .filter_map(|t| t.end_time())
            .reduce(f32::max)
        {
            self.end_time = e;
        }
    }

    pub fn transform_track(&mut self, joint: u32) -> Option<&TransformTrack> {
        let track_index = self.tracks.iter().position(|track| track.id == joint);
        match track_index {
            Some(idx) => Some(&self.tracks[idx]),
            None => {
                self.tracks.push(TransformTrack::new(joint));
                self.tracks.last()
            }
        }
    }

    fn adjust_time_to_fit_range(&self, mut in_time: f32) -> f32 {
        if self.looping {
            if self.duration() <= 0.0 {
                return 0.0;
            }
            in_time = loop_time(in_time, self.start_time, self.end_time);
        } else {
            in_time = clamp(in_time, self.start_time, self.end_time);
        }
        in_time
    }

    fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }
}
