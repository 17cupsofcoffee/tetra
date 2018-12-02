//! Functions and types relating to shader programs.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use error::{Result, TetraError};
use graphics::opengl::GLProgram;
use Context;

/// A shader program, consisting of a vertex shader and a fragment shader.
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
#[derive(Clone)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    /// Creates a new shader program from the given strings.
    pub fn new(ctx: &mut Context, vertex_shader: &str, fragment_shader: &str) -> Shader {
        // TODO: If this fails, we need to actually return an error instead of crashing
        Shader::from_handle(ctx.gl.compile_program(vertex_shader, fragment_shader))
    }

    /// Creates a new shader program from the given files.
    pub fn from_file<P: AsRef<Path>>(
        ctx: &mut Context,
        vertex_path: P,
        fragment_path: P,
    ) -> Result<Shader> {
        Ok(Shader::new(
            ctx,
            &fs::read_to_string(vertex_path).map_err(TetraError::Io)?,
            &fs::read_to_string(fragment_path).map_err(TetraError::Io)?,
        ))
    }

    pub(crate) fn from_handle(handle: GLProgram) -> Shader {
        Shader {
            handle: Rc::new(handle),
        }
    }
}
