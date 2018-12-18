//! Functions and types relating to shader programs.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::graphics::opengl::GLProgram;
use crate::Context;

/// A shader program, consisting of a vertex shader and a fragment shader.
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
#[derive(Clone, PartialEq)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    /// Creates a new shader program from the given files.
    pub fn new<P>(ctx: &mut Context, vertex_path: P, fragment_path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Ok(Shader::from_string(
            ctx,
            &fs::read_to_string(vertex_path)?,
            &fs::read_to_string(fragment_path)?,
        ))
    }

    /// Creates a new shader program from the given strings.
    pub fn from_string(ctx: &mut Context, vertex_shader: &str, fragment_shader: &str) -> Shader {
        // TODO: If this fails, we need to actually return an error instead of crashing
        Shader::from_handle(ctx.gl.compile_program(vertex_shader, fragment_shader))
    }

    pub(crate) fn from_handle(handle: GLProgram) -> Shader {
        Shader {
            handle: Rc::new(handle),
        }
    }
}
