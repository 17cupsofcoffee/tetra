//! Functions and types relating to vector math (provided by the `vek` crate).
//!
//! `vek` is a Rust crate that provides vector math types that are well-documented and
//! convenient to use. Rather than reinventing the wheel, Tetra re-exports the contents
//! of that crate - both for its own internal use, and for you to use in your games.
//!
//! Ideally, the documentation for `vek` would be reproduced here for your convienence.
//! Unfortunately, [rustdoc currently doesn't handle re-exports very well](https://github.com/rust-lang/rust/issues/58693),
//! so this would probably be more confusing than helpful. Until those issues are fixed,
//! you can find the documentation for `vek` by clicking the re-export link below.
//!
//! Note that all of the important types in `vek` (such as `Vec2` and `Mat4`) are
//! re-exported at the top level - you don't need to dig down into the submodules
//! when importing things.

#[doc(no_inline)]
pub use vek::*;
