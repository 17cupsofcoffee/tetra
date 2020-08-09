//! Functions and types relating to measuring and manipulating time.

use std::collections::VecDeque;

use std::time::{Duration, Instant};

use crate::Context;

/// The different timestep modes that a game can have.
///
/// # Serde
///
/// Serialization and deserialization of this type (via [Serde](https://serde.rs/))
/// can be enabled via the `serde_support` feature.
#[derive(Debug, Copy, Clone)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum Timestep {
    /// In fixed timestep mode, updates will happen at a consistent rate (the `f64` value in the enum
    /// variant representing the number of times per second), while rendering will happen as fast as
    /// the hardware (and vsync settings) will allow.
    ///
    /// This has the advantage of making your game's updates deterministic, so they will act the same
    /// on hardware of different speeds. It also means that your update code does not need to use
    /// `time::get_delta_time` to integrate the amount of time passed into your calculations. However,
    /// it can lead to some slight stutter if your rendering code does not account for the possibility
    /// of updating and rendering to be out of sync with each other.
    ///
    /// To avoid stutter, you should interpolate your rendering using `time::get_blend_factor`. The
    /// [`interpolation`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/interpolation.rs)
    /// example in the Tetra repository shows some different approaches to doing this.
    ///
    /// This mode is currently the default.
    Fixed(f64),

    /// In variable timestep mode, updates and rendering will happen in lockstep, one after the other,
    /// as fast as the hardware (and vsync settings) will allow.
    ///
    /// This has the advantage of being simple to reason about (updates can never happen multiple times
    /// or get skipped), but is not deterministic, so your updates may not act the same on every
    /// run of the game loop.
    ///
    /// To integrate the amount of time that has passed into your game's calculations, use
    /// `time::get_delta_time`.
    Variable,
}

struct FixedTimeStepState {
    ticks_per_second: f64,
    tick_rate: Duration,
    accumulator: Duration,
}

pub(crate) struct TimeContext {
    timestep: Option<FixedTimeStepState>,
    fps_tracker: VecDeque<f64>,
    last_time: Instant,
    elapsed: Duration,
}

impl TimeContext {
    pub(crate) fn new(timestep: Timestep) -> TimeContext {
        // We fill the buffer with values so that the FPS counter doesn't jitter
        // at startup.
        let mut fps_tracker = VecDeque::with_capacity(200);
        fps_tracker.resize(200, 1.0 / 60.0);

        TimeContext {
            timestep: create_timestep_state(timestep),
            fps_tracker,
            last_time: Instant::now(),
            elapsed: Duration::from_secs(0),
        }
    }
}

pub(crate) fn reset(ctx: &mut Context) {
    ctx.time.last_time = Instant::now();

    if let Some(fixed) = &mut ctx.time.timestep {
        fixed.accumulator = Duration::from_secs(0);
    }
}

pub(crate) fn tick(ctx: &mut Context) {
    let current_time = Instant::now();
    ctx.time.elapsed = current_time - ctx.time.last_time;
    ctx.time.last_time = current_time;

    if let Some(fixed) = &mut ctx.time.timestep {
        fixed.accumulator += ctx.time.elapsed;
    }

    // Since we fill the buffer when we create the context, we can cycle it
    // here and it shouldn't reallocate.
    ctx.time.fps_tracker.pop_front();
    ctx.time
        .fps_tracker
        .push_back(ctx.time.elapsed.as_secs_f64());
}

pub(crate) fn is_fixed_update_ready(ctx: &mut Context) -> bool {
    match &mut ctx.time.timestep {
        Some(fixed) if fixed.accumulator >= fixed.tick_rate => {
            fixed.accumulator -= fixed.tick_rate;
            true
        }
        _ => false,
    }
}

/// Returns the amount of time that has passed since the last frame was rendered.
///
/// When using a variable time step, you should use this to integrate the amount of time that
/// has passed into your game's calculations. For example, if you wanted to move a `Vec2` 32
/// units to the right per second, you would do
/// `foo.y += 32.0 * time::get_delta_time(ctx).as_secs_f32()`
///
/// When using a fixed time step, the above still applies, but only to rendering - you should
/// not integrate the delta time into your update calculations.
pub fn get_delta_time(ctx: &Context) -> Duration {
    ctx.time.elapsed
}

/// Returns the amount of time that has accumulated between updates.
///
/// When using a fixed time step, as time passes, this value will increase;
/// as updates occur, it will decrease.
///
/// When using a variable time step, this function always returns `Duration::from_secs(0)`.
pub fn get_accumulator(ctx: &Context) -> Duration {
    match &ctx.time.timestep {
        Some(fixed) => fixed.accumulator,
        None => Duration::from_secs(0),
    }
}

/// Returns a value between 0.0 and 1.0, representing how far between updates the game loop
/// currently is.
///
/// For example, if the value is 0.01, an update just happened; if the value is 0.99,
/// an update is about to happen.
///
/// This can be used to interpolate when rendering.
///
/// This function returns an f32, which is usually what you want when blending - however,
/// if you need a more precise representation of the blend factor, you can call
/// `time::get_blend_factor_precise`.
pub fn get_blend_factor(ctx: &Context) -> f32 {
    match &ctx.time.timestep {
        Some(fixed) => fixed.accumulator.as_secs_f32() / fixed.tick_rate.as_secs_f32(),
        None => 0.0,
    }
}

/// Returns a precise value between 0.0 and 1.0, representing how far between updates the game loop
/// currently is.
///
/// For example, if the value is 0.01, an update just happened; if the value is 0.99,
/// an update is about to happen.
///
/// This can be used to interpolate when rendering.
///
/// This function returns an f64, which is a very precise representation of the blend factor,
/// but often difficult to use in game logic without casting. If you need an `f32`, call
/// `time::get_blend_factor` instead.
pub fn get_blend_factor_precise(ctx: &Context) -> f64 {
    match &ctx.time.timestep {
        Some(fixed) => fixed.accumulator.as_secs_f64() / fixed.tick_rate.as_secs_f64(),
        None => 0.0,
    }
}

/// Gets the current timestep of the application.
pub fn get_timestep(ctx: &Context) -> Timestep {
    match &ctx.time.timestep {
        Some(fixed) => Timestep::Fixed(fixed.ticks_per_second),
        None => Timestep::Variable,
    }
}

/// Sets the timestep of the application.
pub fn set_timestep(ctx: &mut Context, timestep: Timestep) {
    ctx.time.timestep = create_timestep_state(timestep);
}

fn create_timestep_state(timestep: Timestep) -> Option<FixedTimeStepState> {
    match timestep {
        Timestep::Fixed(ticks_per_second) => Some(FixedTimeStepState {
            ticks_per_second,
            tick_rate: Duration::from_secs_f64(1.0 / ticks_per_second),
            accumulator: Duration::from_secs(0),
        }),
        Timestep::Variable => None,
    }
}

/// Returns the current frame rate, averaged out over the last 200 frames.
pub fn get_fps(ctx: &Context) -> f64 {
    1.0 / (ctx.time.fps_tracker.iter().sum::<f64>() / ctx.time.fps_tracker.len() as f64)
}
