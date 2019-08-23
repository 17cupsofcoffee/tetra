//! The platform abstraction used for windowing, input and creating the GL context.
//!
//! All code interacting with SDL or the browser must be placed within this module. This
//! is to facilitate creating alternate backends in the future.
//!
//! The interface for this module is *not* stable, and will likely not be made public
//! in its current form.

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::*;

#[cfg(target_arch = "wasm32")]
mod web;

#[cfg(target_arch = "wasm32")]
pub use web::*;
