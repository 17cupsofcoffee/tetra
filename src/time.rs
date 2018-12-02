//! Functions and types relating to measuring time.

use std::time::Duration;

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
