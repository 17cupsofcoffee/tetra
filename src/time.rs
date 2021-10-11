//! Functions and types relating to measuring and manipulating time.

use std::collections::VecDeque;

use std::time::Duration;

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
    /// on hardware of different speeds. However, it can lead to some slight stutter if your
    /// rendering code does not account for the possibility for updating and rendering to be
    /// out of sync with each other.
    ///
    /// To avoid stutter, you should interpolate your rendering using [`get_blend_factor`]. The
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
    /// [`get_delta_time`].
    Variable,
}

pub(crate) struct FpsTracker {
    buffer: VecDeque<f64>,
}

impl FpsTracker {
    fn new() -> FpsTracker {
        FpsTracker {
            buffer: VecDeque::with_capacity(200),
        }
    }

    pub(crate) fn push(&mut self, frame_time: Duration) {
        if self.buffer.len() == 200 {
            self.buffer.pop_front();
        }

        self.buffer.push_back(frame_time.as_secs_f64());
    }

    fn get_fps(&self) -> f64 {
        1.0 / (self.buffer.iter().sum::<f64>() / self.buffer.len() as f64)
    }
}

pub(crate) struct TimeContext {
    pub(crate) fps_tracker: FpsTracker,
    pub(crate) ticks_per_second: Option<f64>,
    pub(crate) tick_rate: Option<Duration>,
    pub(crate) delta_time: Duration,
    pub(crate) accumulator: Duration,
}

impl TimeContext {
    pub(crate) fn new(timestep: Timestep) -> TimeContext {
        let ticks_per_second = match timestep {
            Timestep::Fixed(tps) => Some(tps),
            Timestep::Variable => None,
        };

        let tick_rate = match timestep {
            Timestep::Fixed(tps) => Some(Duration::from_secs_f64(1.0 / tps)),
            Timestep::Variable => None,
        };

        TimeContext {
            fps_tracker: FpsTracker::new(),
            ticks_per_second,
            tick_rate,
            delta_time: Duration::from_secs(0),
            accumulator: Duration::from_secs(0),
        }
    }
}

pub(crate) fn reset(ctx: &mut Context) {
    ctx.time.delta_time = Duration::from_secs(0);
    ctx.time.accumulator = Duration::from_secs(0);
}

/// Returns the amount of time that has passed since the last update or draw.
///
/// This can be used to integrate the amount of time that has passed into your game's
/// calculations. For example, if you wanted to move a [`Vec2`](crate::math::Vec2) 32
/// units to the right per second, you could do
/// `foo.y += 32.0 * time::get_delta_time(ctx).as_secs_f32()`.
///
/// When using a fixed time step, calling this function during an update will always
/// return the configured update rate. This is to prevent floating point error/non-determinism
/// from creeping into your game's calculations!
pub fn get_delta_time(ctx: &Context) -> Duration {
    ctx.time.delta_time
}

/// Returns the amount of time that has accumulated between updates.
///
/// When using a fixed time step, as time passes, this value will increase;
/// as updates occur, it will decrease.
///
/// When using a variable time step, this function always returns `Duration::from_secs(0)`.
pub fn get_accumulator(ctx: &Context) -> Duration {
    ctx.time.accumulator
}

/// Returns a value between 0.0 and 1.0, representing how far between updates the game loop
/// currently is.
///
/// For example, if the value is 0.01, an update just happened; if the value is 0.99,
/// an update is about to happen.
///
/// This can be used to interpolate when rendering.
///
/// This function returns an [`f32`], which is usually what you want when blending - however,
/// if you need a more precise representation of the blend factor, you can call
/// [`get_blend_factor_precise`].
pub fn get_blend_factor(ctx: &Context) -> f32 {
    match ctx.time.tick_rate {
        Some(tick_rate) => ctx.time.accumulator.as_secs_f32() / tick_rate.as_secs_f32(),
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
/// This function returns an [`f64`], which is a very precise representation of the blend factor,
/// but often difficult to use in game logic without casting. If you need an [`f32`], call
/// [`get_blend_factor`] instead.
pub fn get_blend_factor_precise(ctx: &Context) -> f64 {
    match ctx.time.tick_rate {
        Some(tick_rate) => ctx.time.accumulator.as_secs_f64() / tick_rate.as_secs_f64(),
        None => 0.0,
    }
}

/// Gets the current timestep of the application.
pub fn get_timestep(ctx: &Context) -> Timestep {
    match ctx.time.ticks_per_second {
        Some(tps) => Timestep::Fixed(tps),
        None => Timestep::Variable,
    }
}

/// Sets the timestep of the application.
pub fn set_timestep(ctx: &mut Context, timestep: Timestep) {
    ctx.time.ticks_per_second = match timestep {
        Timestep::Fixed(tps) => Some(tps),
        Timestep::Variable => None,
    };

    ctx.time.tick_rate = match timestep {
        Timestep::Fixed(tps) => Some(Duration::from_secs_f64(1.0 / tps)),
        Timestep::Variable => None,
    };
}

/// Returns the current frame rate, averaged out over the last 200 frames.
pub fn get_fps(ctx: &Context) -> f64 {
    ctx.time.fps_tracker.get_fps()
}
