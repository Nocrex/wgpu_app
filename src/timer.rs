use std::{time::Instant};

pub struct Timer {
    last: Instant,
    fps: u32,
    last_delta: f32,
    tick_duration: f32,
    frame_count: u32,
    frame_time: f32,
    fps_update_time: f32,

    abs_time: f32,
}

/// Keeps track of timing
impl Timer {
    pub fn new() -> Timer {
        Timer {
            last: Instant::now(),
            fps: 0,
            last_delta: 0.0,
            tick_duration: 0.001,
            frame_count: 0,
            frame_time: 0.0,
            fps_update_time: 0.25,

            abs_time: 0.0,
        }
    }

    /// Reset time to 0
    pub fn reset(&mut self) {
        self.last = Instant::now();
        self.abs_time = 0.0;
    }

    /// Returns the time since `go()` last returned a value
    /// If less than `frame_min_duration` has elapsed since this function last returned a value then it will return None, 
    /// indicating it is not yet time for the next tick. Otherwise it will return `Some` containing how much time has elapsed in seconds 
    pub fn go(&mut self) -> Option<f32> {
        let now = self.last.elapsed();
        let delta = (now.as_micros() as f32) / 1_000_000.0;

        if delta < self.tick_duration {
            return None;
        }

        self.abs_time += self.last_delta;

        self.frame_count += 1;
        self.frame_time += delta;
        if self.frame_time > self.fps_update_time {
            self.fps = (self.frame_count as f32 * (1.0 / self.frame_time)) as u32;
            self.frame_count = 0;
            self.frame_time = 0.0;
        }

        self.last_delta = delta;
        self.last = Instant::now();
        Some(delta)
    }

    /// Set how many seconds should pass before the next tick
    pub fn set_tick_duration(&mut self, dur: f32) {
        self.tick_duration = dur;
    }

    /// Set how often the fps count should be updated, shorter durations update the fps count more often, but slower durations are generally more consistent and accurate
    pub fn set_fps_update_time(&mut self, dur: f32) {
        self.fps_update_time = dur;
    }

    /// Approximate FPS
    pub fn fps(&self) -> u32 {
        self.fps
    }

    /// How much time has passed between ticks (same as the value from calling `go` and is only updated whenever `go` succeeds).
    pub fn delta(&self) -> f32 {
        self.last_delta
    }

    /// How much time has passed since this Timer was created or `reset` was last called
    pub fn absolute_time(&self) -> f32 {
        self.abs_time
    }
}
