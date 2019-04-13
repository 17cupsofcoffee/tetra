//! Functions and types relating to measuring and manipulating time.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::Context;

pub(crate) struct TimeContext {
    tick_rate: Duration,
    fps_tracker: VecDeque<f64>,

    last_time: Instant,
    lag: Duration,
}

impl TimeContext {
    pub(crate) fn new(tick_rate: f64) -> TimeContext {
        // We fill the buffer with values so that the FPS counter doesn't jitter
        // at startup.
        let mut fps_tracker = VecDeque::with_capacity(200);
        fps_tracker.resize(200, 1.0 / 60.0);

        TimeContext {
            tick_rate: f64_to_duration(tick_rate),
            fps_tracker,
            last_time: Instant::now(),
            lag: Duration::from_secs(0),
        }
    }
}

pub(crate) fn reset(ctx: &mut Context) {
    ctx.time.last_time = Instant::now();
    ctx.time.lag = Duration::from_secs(0);
}

pub(crate) fn tick(ctx: &mut Context) {
    let current_time = Instant::now();
    let elapsed = current_time - ctx.time.last_time;
    ctx.time.last_time = current_time;
    ctx.time.lag += elapsed;

    // Since we fill the buffer when we create the context, we can cycle it
    // here and it shouldn't reallocate.
    ctx.time.fps_tracker.pop_front();
    ctx.time.fps_tracker.push_back(duration_to_f64(elapsed));
}

pub(crate) fn is_tick_ready(ctx: &Context) -> bool {
    ctx.time.lag >= ctx.time.tick_rate
}

pub(crate) fn consume_tick(ctx: &mut Context) {
    ctx.time.lag -= ctx.time.tick_rate;
}

// TODO: What's the proper name for the interpolation amount? NAMING AGH
pub(crate) fn get_alpha(ctx: &Context) -> f64 {
    duration_to_f64(ctx.time.lag) / duration_to_f64(ctx.time.tick_rate)
}

/// Converts a `std::time::Duration` to an `f64`. This is less accurate, but
/// usually more useful.
pub fn duration_to_f64(duration: Duration) -> f64 {
    let seconds = duration.as_secs() as f64;
    let nanos = f64::from(duration.subsec_nanos()) * 1e-9;
    seconds + nanos
}

/// Converts an `f64` to a `std::time::Duration`.
pub fn f64_to_duration(duration: f64) -> Duration {
    debug_assert!(duration >= 0.0);
    let seconds = duration.trunc() as u64;
    let nanos = (duration.fract() * 1e9) as u32;
    Duration::new(seconds, nanos)
}

/// Gets the update tick rate of the application, in ticks per second.
pub fn get_tick_rate(ctx: &Context) -> f64 {
    1.0 / duration_to_f64(ctx.time.tick_rate)
}

/// Sets the update tick rate of the application, in ticks per second.
pub fn set_tick_rate(ctx: &mut Context, tick_rate: f64) {
    ctx.time.tick_rate = f64_to_duration(1.0 / tick_rate);
}

/// Returns the current frame rate, averaged out over the last 200 frames.
pub fn get_fps(ctx: &Context) -> f64 {
    1.0 / (ctx.time.fps_tracker.iter().sum::<f64>() / ctx.time.fps_tracker.len() as f64)
}
