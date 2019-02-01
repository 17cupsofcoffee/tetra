//! Functions and types relating to measuring and manipulating time.

use std::time::Duration;

use crate::Context;

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
    1.0 / duration_to_f64(ctx.tick_rate)
}

/// Sets the update tick rate of the application, in ticks per second.
pub fn set_tick_rate(ctx: &mut Context, tick_rate: f64) {
    ctx.tick_rate = f64_to_duration(1.0 / tick_rate);
}

/// Returns the current frame rate, averaged out over the last 200 frames.
pub fn get_fps(ctx: &Context) -> f64 {
    1.0 / (ctx.fps_tracker.iter().sum::<f64>() / ctx.fps_tracker.len() as f64)
}
