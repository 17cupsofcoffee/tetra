use std::time::Duration;

use glm::Mat4;

pub fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    // Taken from GGEZ - nalgebra doesn't like upside-down projections
    let c0r0 = 2.0 / (right - left);
    let c0r1 = 0.0;
    let c0r2 = 0.0;
    let c0r3 = 0.0;
    let c1r0 = 0.0;
    let c1r1 = 2.0 / (top - bottom);
    let c1r2 = 0.0;
    let c1r3 = 0.0;
    let c2r0 = 0.0;
    let c2r1 = 0.0;
    let c2r2 = -2.0 / (far - near);
    let c2r3 = 0.0;
    let c3r0 = -(right + left) / (right - left);
    let c3r1 = -(top + bottom) / (top - bottom);
    let c3r2 = -(far + near) / (far - near);
    let c3r3 = 1.0;

    Mat4::from([
        [c0r0, c0r1, c0r2, c0r3],
        [c1r0, c1r1, c1r2, c1r3],
        [c2r0, c2r1, c2r2, c2r3],
        [c3r0, c3r1, c3r2, c3r3],
    ])
}

pub fn duration_to_f64(duration: Duration) -> f64 {
    let seconds = duration.as_secs() as f64;
    let nanos = f64::from(duration.subsec_nanos()) * 1e-9;
    seconds + nanos
}

pub fn f64_to_duration(duration: f64) -> Duration {
    debug_assert!(duration >= 0.0);
    let seconds = duration.trunc() as u64;
    let nanos = (duration.fract() * 1e9) as u32;
    Duration::new(seconds, nanos)
}
