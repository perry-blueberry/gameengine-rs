use num_traits::clamp;

use super::{pose::Pose, track::loop_time, transform_track::TransformTrack};

#[derive(Clone)]
pub struct Clip {
    tracks: Vec<TransformTrack>,
    pub name: String,
    pub start_time: f32,
    end_time: f32,
    pub looping: bool,
}

impl Clip {
    pub fn new(name: Option<&str>) -> Self {
        Self {
            tracks: vec![],
            name: name.unwrap_or("No name given").to_owned(),
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
            let animated = track.sample(local.clone(), time, self.looping);
            out_pose.set_local_transform(joint, animated);
        }
        time
    }

    pub fn recalculate_duration(&mut self) {
        if let Some(s) = self
            .tracks
            .iter()
            .filter(|t| t.is_valid())
            .filter_map(|t| t.start_time())
            .reduce(f32::min)
        {
            self.start_time = s;
        } else {
            self.start_time = 0.0;
        }
        if let Some(e) = self
            .tracks
            .iter()
            .filter(|t| t.is_valid())
            .filter_map(|t| t.end_time())
            .reduce(f32::max)
        {
            self.end_time = e;
        } else {
            self.end_time = 0.0;
        }
    }

    pub fn transform_track(&mut self, joint: u32) -> &mut TransformTrack {
        let track_index = self.tracks.iter().position(|track| track.id == joint);
        match track_index {
            Some(idx) => &mut self.tracks[idx],
            None => {
                self.tracks.push(TransformTrack::new(joint));
                self.tracks.last_mut().unwrap()
            }
        }
    }

    pub fn add_track(&mut self, track: TransformTrack) {
        self.tracks.push(track);
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

    pub fn duration(&self) -> f32 {
        self.end_time - self.start_time
    }
}
