//! Functions and types relating to math (provided by the nalgebra-glm crate).
//!
//! Ideally, the documentation for `nalgebra-glm` would be reproduced here for your convienence. Unfortunately,
//! [rustdoc currently doesn't handle this very well](https://github.com/rust-lang/rust/issues/15823), and the
//! re-exported docs would likely be more confusing than helpful. Until those issues are fixed, you can find
//! the documentation for `nalgebra-glm` by clicking the re-export link below.
//!
//! # Commonly Used Types
//!
//! * `Vec2`, `Vec3` and `Vec4` are 2D vectors containing `f32` values.
//! * `Mat2`, `Mat3` and `Mat4` are matrices containing `f32` values.
//!
//! # Commonly Used Functions
//!
//! * `abs` returns the absolute value of a vector or matrix (i.e. it turns minus numbers into positive numbers).
//! * `ceil` rounds the components of a vector or matrix *up* to an integer.
//! * `clamp` clamps the components of a vector or matrix between a min and max value.
//! * `clamp_scalar` clamps a number between a min and max value.
//! * `clamp_vec` clamps a vector between a min and max vector.
//! * `distance` returns the distance between two points.
//! * `distance2` returns the distance between two points, squared.
//! * `floor` rounds the components of a vector or a matrix *down* to an integer.
//! * `fract` returns the fractional part of the components of a vector or matrix.
//! * `length` and `magnitude` return the magnitude of a vector.
//! * `length2` and `magnitude2` return the magnitude of a vector, squared.
//! * `lerp` and `mix` return the linear interpolation of two vectors, using a number as the blend amount.
//! * `lerp_scalar` and `mix_scalar` return the linear interpolation of two numbers.
//! * `lerp_vec` and `mix_vec` return the linear interpolation of two vectors, using a vector as the blend amount.
//! * `ortho` returns an orthographic projection matrix.
//! * `perspective` returns a perpspective projection matrix.
//! * `radians` converts the components of a vector or matrix from degrees to radians.
//! * `round` rounds the components of a vector or matrix to the nearest integer.
//! * `trunc` truncates the components of a vector or matrix to integers.

#[doc(no_inline)]
pub use nalgebra_glm::*;
