use std::ops::{Add, Mul};

use glam::{Quat, Vec3};
use num_traits::clamp;

use super::{
    array_type::ArrayType,
    frame::Frame,
    interpolation::Interpolation,
    track_helpers::{AdjustHermiteResult, Interpolate, Neighborhood},
};

pub type ScalarTrack = Track<f32>;
pub type Vector3Track = Track<Vec3>;
pub type QuatTrack = Track<Quat>;

#[derive(Debug, Clone)]
pub struct Track<T: ArrayType> {
    pub frames: Vec<Frame<T>>,
    interp: Interpolation,
}

impl<T> Track<T>
where
    T: Neighborhood
        + AdjustHermiteResult
        + Copy
        + Mul<f32, Output = T>
        + Add<Output = T>
        + Default
        + ArrayType
        + Interpolate,
{
    pub fn new() -> Self {
        Self {
            frames: vec![],
            interp: Interpolation::Linear,
        }
    }

    pub fn new_with_args(interp: Interpolation, frames: Vec<Frame<T>>) -> Self {
        Self { frames, interp }
    }

    pub fn start_time(&self) -> Option<f32> {
        Some(self.frames.first()?.time)
    }

    pub fn end_time(&self) -> Option<f32> {
        Some(self.frames.last()?.time)
    }

    fn hermite(t: f32, point1: &T, slope1: &T, point2: &T, slope2: &T) -> T {
        let tt = t * t;
        let ttt = tt * t;

        let mut p2 = *point2;
        point1.neighborhood(&mut p2);

        let h1 = 2.0 * ttt - 3.0 * tt + 1.0;
        let h2 = -2.0 * ttt + 3.0 * tt;
        let h3 = ttt - 2.0 * tt + t;
        let h4 = ttt - tt;

        let result: T = *point1 * h1 + p2 * h2 + *slope1 * h3 + *slope2 * h4;
        result.adjust_hermite_result()
    }

    pub fn sample(&self, t: f32, looping: bool) -> T {
        match self.interp {
            Interpolation::Constant => self.sample_constant(t, looping),
            Interpolation::Linear => self.sample_linear(t, looping),
            Interpolation::Cubic => self.sample_cubic(t, looping),
        }
    }

    fn sample_constant(&self, t: f32, looping: bool) -> T {
        match self.frame_index(t, looping) {
            Some(i) => T::from_slice(&self.frames[i].value),
            _ => T::default(),
        }
    }

    fn sample_linear(&self, t: f32, looping: bool) -> T {
        match self.frame_index(t, looping) {
            Some(this_frame) if this_frame < self.frames.len() - 1 => {
                let next_frame = this_frame + 1;
                let track_time = self.adjust_time_to_fit_track(t, looping);
                let this_frame_time = self.frames[this_frame].time;
                let frame_delta = self.frames[next_frame].time - this_frame_time;
                if frame_delta <= 0.0 {
                    return T::default();
                }
                let t = (track_time - this_frame_time) / frame_delta;
                let start = T::from_slice(&self.frames[this_frame].value);
                let end = T::from_slice(&self.frames[next_frame].value);
                start.interpolate(&end, t)
            }
            _ => T::default(),
        }
    }

    fn sample_cubic(&self, t: f32, looping: bool) -> T {
        match self.frame_index(t, looping) {
            Some(this_frame) if this_frame < self.frames.len() - 1 => {
                let next_frame = this_frame + 1;
                let track_time = self.adjust_time_to_fit_track(t, looping);
                let this_frame_time = self.frames[this_frame].time;
                let frame_delta = self.frames[next_frame].time - this_frame_time;
                if frame_delta <= 0.0 {
                    return T::default();
                }
                let t = (track_time - this_frame_time) / frame_delta;

                let point1 = T::from_slice(&self.frames[this_frame].value);
                let slope1 = T::from_slice(&self.frames[this_frame].out_tangent) * frame_delta;

                let point2 = T::from_slice(&self.frames[next_frame].value);
                let slope2 = T::from_slice(&self.frames[next_frame].in_tangent) * frame_delta;

                Self::hermite(t, &point1, &slope1, &point2, &slope2)
            }
            _ => T::default(),
        }
    }

    pub fn interpolation(&self) -> Interpolation {
        self.interp
    }

    pub fn set_interpolation(&mut self, interp: Interpolation) {
        self.interp = interp;
    }

    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn frame(&self, idx: usize) -> &Frame<T> {
        &self.frames[idx]
    }

    fn frame_index(&self, mut t: f32, looping: bool) -> Option<usize> {
        if self.frames.len() < 2 {
            return None;
        }
        let start_time = self.frames[0].time;
        if looping {
            let end_time = self.frames[self.frames.len() - 1].time;
            t = loop_time(t, start_time, end_time);
        } else {
            // If time is before or at the first frame, return 0
            if t <= start_time {
                return Some(0);
            }
            // If time is at or after the second-to-last frame, return the index of the second-to-last frame
            if t >= self.frames[self.frames.len() - 2].time {
                return Some(self.frames.len() - 2);
            }
        }

        // Find the index of the frame at or before the given time
        for (idx, frame) in self.frames.iter().enumerate().rev() {
            if t >= frame.time {
                return Some(idx);
            }
        }
        None
    }

    fn adjust_time_to_fit_track(&self, mut t: f32, looping: bool) -> f32 {
        if self.frames.is_empty() {
            return 0.0;
        }

        let start_time = self.start_time().unwrap_or_default();
        let end_time = self.end_time().unwrap_or_default();
        if end_time - start_time <= 0.0 {
            return 0.0;
        }
        if looping {
            t = loop_time(t, start_time, end_time);
        } else {
            t = clamp(t, start_time, end_time);
        }
        t
    }
}

pub fn loop_time(mut t: f32, start_time: f32, end_time: f32) -> f32 {
    let duration = end_time - start_time;
    // Wrap the time value within the duration of the frames
    t = (t - start_time) % duration;
    if t < 0.0 {
        t += duration;
    }
    t + start_time
}
