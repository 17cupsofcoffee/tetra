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
///
/// # Attributes and Uniforms
///
/// Tetra's shaders currently take the following parameters:
///
/// * The `a_position` attribute is a `vec2`, representing the vertex's position in pixel co-ordinates.
/// * The `a_uv` attribute is a `vec2`, representing the texture co-ordinates that should be used for the vertex.
/// * The `a_color` attribute is a `vec4`, representing a color that the output should be multiplied by.
/// * The `u_projection` uniform is a `mat4`, which can be used to project the vertex's position into GL co-ordinates.
/// * The `u_texture` uniform is a `sampler2D`, which can be used to sample colors from the current texture.
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
}
