//! Functions and types relating to math (provided by the nalgebra-glm crate).
//!
//! This module re-exports the contents of the `nalgebra-glm` crate.
//!
//! Ideally, the documentation for `nalgebra-glm` would be reproduced here for your convienence. Unfortunately,
//! [rustdoc currently doesn't handle this very well](https://github.com/rust-lang/rust/issues/15823), and the
//! re-exported docs would likely be more confusing than helpful. Until those issues are fixed, you can find
//! the documentation for `nalgebra-glm` on [their website](https://www.nalgebra.org/rustdoc_glm/nalgebra_glm/index.html).

#[doc(no_inline)]
pub use nalgebra_glm::*;
