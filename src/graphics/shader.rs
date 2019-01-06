//! Functions and types relating to shader programs.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::graphics::opengl::GLProgram;
use crate::Context;

#[doc(inline)]
pub use crate::graphics::opengl::UniformValue;

/// The default vertex shader.
pub static DEFAULT_VERTEX_SHADER: &str = include_str!("../resources/shader.vert");

/// The default fragment shader.
pub static DEFAULT_FRAGMENT_SHADER: &str = include_str!("../resources/shader.frag");

/// A shader program, consisting of a vertex shader and a fragment shader.
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
#[derive(Debug, Clone, PartialEq)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    /// Creates a new shader program from the given files.
    pub fn new<P>(ctx: &mut Context, vertex_path: P, fragment_path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::from_string(
            ctx,
            &fs::read_to_string(vertex_path)?,
            &fs::read_to_string(fragment_path)?,
        )
    }

    /// Creates a new shader program from the given vertex shader file.
    ///
    /// The default fragment shader will be used.
    pub fn vertex<P>(ctx: &mut Context, path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::from_string(ctx, &fs::read_to_string(path)?, DEFAULT_FRAGMENT_SHADER)
    }

    /// Creates a new shader program from the given fragment shader file.
    ///
    /// The default vertex shader will be used.
    pub fn fragment<P>(ctx: &mut Context, path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::from_string(ctx, DEFAULT_VERTEX_SHADER, &fs::read_to_string(path)?)
    }

    /// Creates a new shader program from the given strings.
    pub fn from_string(
        ctx: &mut Context,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Shader> {
        ctx.gl
            .compile_program(vertex_shader, fragment_shader)
            .map(Shader::from_handle)
    }

    pub(crate) fn from_handle(handle: GLProgram) -> Shader {
        Shader {
            handle: Rc::new(handle),
        }
    }

    /// Sets the value of the specifed uniform parameter.
    pub fn set_uniform<V>(&mut self, ctx: &mut Context, name: &str, value: V)
    where
        V: UniformValue,
    {
        ctx.gl.set_uniform(&self.handle, name, value);
    }
}
