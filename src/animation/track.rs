use std::{
    marker::PhantomData,
    ops::{Add, Mul},
};

use cgmath::{Quaternion, Vector3};

use super::{
    array_type::ArrayType,
    frame::Frame,
    interpolation::Interpolation,
    track_helpers::{AdjustHermiteResult, Neighborhood},
};

pub(crate) type ScalarTrack = Track<f32>;
pub(crate) type Vector3Track = Track<Vector3<f32>>;
pub(crate) type QuatTrack = Track<Quaternion<f32>>;

pub(crate) struct Track<T: ArrayType> {
    frames: Vec<Frame<T>>,
    interp: Interpolation,
    _phantom: PhantomData<T>,
}

impl<T> Track<T>
where
    T: Neighborhood
        + AdjustHermiteResult
        + Copy
        + Mul<f32, Output = T>
        + Add<Output = T>
        + Default
        + ArrayType,
{
    pub(crate) fn new() -> Self {
        Self {
            frames: vec![],
            interp: Interpolation::Linear,
            _phantom: PhantomData,
        }
    }

    pub(crate) fn start_time(&self) -> Option<f32> {
        Some(self.frames.first()?.time)
    }

    pub(crate) fn end_time(&self) -> Option<f32> {
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

    fn sample_constant(&self, t: f32, looping: bool) -> T {
        match self.frame_index(t, looping) {
            Some(i) => T::from_slice(&self.frames[i].value),
            _ => T::default(),
        }
    }

    fn frame_index(&self, mut t: f32, looping: bool) -> Option<usize> {
        if self.frames.len() < 2 {
            return None;
        }
        if looping {
            let start_time = self.start_time()?;
            let end_time = self.end_time()?;
            let duration = end_time - start_time;

            // Wrap the time value within the duration of the frames
            t = (t - start_time) % duration;
            if t <= 0.0 {
                t += duration;
            }
            t = t + start_time;
        } else {
            // If time is before or at the first frame, return 0
            if t <= self.frames[0].time {
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
}
