//! Functions and types relating to shader programs.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::graphics::opengl::{GLDevice, GLProgram};
use crate::Context;

#[doc(inline)]
pub use crate::graphics::opengl::UniformValue;

/// The default vertex shader.
///
/// The source code for this shader is available in [`src/resources/shader.vert`](https://github.com/17cupsofcoffee/tetra/blob/master/src/resources/shader.vert).
pub static DEFAULT_VERTEX_SHADER: &str = include_str!("../resources/shader.vert");

/// The default fragment shader.
///
/// The source code for this shader is available in [`src/resources/shader.vert`](https://github.com/17cupsofcoffee/tetra/blob/master/src/resources/shader.frag).
pub static DEFAULT_FRAGMENT_SHADER: &str = include_str!("../resources/shader.frag");

/// A shader program, consisting of a vertex shader and a fragment shader.
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
///
/// # Vertex Shaders
///
/// Vertex shaders take in data via three attributes:
///
/// * `a_position` - A `vec2` representing the position of the vertex in world space.
/// * `a_uv` - A `vec2` representing the texture co-ordinates that are associated with the vertex.
/// * `a_color` - A `vec4` representing a color to multiply the output by.
///
/// # Fragment Shaders
///
/// Fragment shaders have a single `vec4` output called `o_color` - this should be set to the desired output color for the
/// fragment.
///
/// # Uniforms
///
/// By default, the shader is provided with two uniform variables:
///
/// * `u_projection` - A `mat4` which can be used to translate world space co-ordinates into screen space.
/// * `u_texture` - A `sampler2D` which can be used to access color data from the currently active texture.
///
/// You can also set data into your own uniform variables via the `set_uniform` method.
#[derive(Debug, Clone, PartialEq)]
pub struct Shader {
    pub(crate) handle: Rc<GLProgram>,
}

impl Shader {
    /// Creates a new shader program from the given files.
    ///
    /// # Errors
    ///
    /// If the file could not be read, a `TetraError::Io` will be returned.
    ///
    /// If the shader could not be compiled, a `TetraError::OpenGl` will be returned.
    pub fn new<P>(ctx: &mut Context, vertex_path: P, fragment_path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::with_device(
            &mut ctx.gl,
            &fs::read_to_string(vertex_path)?,
            &fs::read_to_string(fragment_path)?,
        )
    }

    /// Creates a new shader program from the given vertex shader file.
    ///
    /// The default fragment shader will be used.
    ///
    /// # Errors
    ///
    /// If the file could not be read, a `TetraError::Io` will be returned.
    ///
    /// If the shader could not be compiled, a `TetraError::OpenGl` will be returned.
    pub fn vertex<P>(ctx: &mut Context, path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::with_device(
            &mut ctx.gl,
            &fs::read_to_string(path)?,
            DEFAULT_FRAGMENT_SHADER,
        )
    }

    /// Creates a new shader program from the given fragment shader file.
    ///
    /// The default vertex shader will be used.
    ///
    /// # Errors
    ///
    /// If the file could not be read, a `TetraError::Io` will be returned.
    ///
    /// If the shader could not be compiled, a `TetraError::OpenGl` will be returned.
    pub fn fragment<P>(ctx: &mut Context, path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::with_device(
            &mut ctx.gl,
            DEFAULT_VERTEX_SHADER,
            &fs::read_to_string(path)?,
        )
    }

    /// Creates a new shader program from the given strings.
    ///
    /// # Errors
    ///
    /// If the shader could not be compiled, a `TetraError::OpenGl` will be returned.
    pub fn from_string(
        ctx: &mut Context,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Shader> {
        Shader::with_device(&mut ctx.gl, vertex_shader, fragment_shader)
    }

    pub(crate) fn with_device(
        device: &mut GLDevice,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Shader> {
        let handle = device.compile_program(vertex_shader, fragment_shader)?;

        Ok(Shader {
            handle: Rc::new(handle),
        })
    }

    /// Sets the value of the specifed uniform parameter.
    pub fn set_uniform<V>(&mut self, ctx: &mut Context, name: &str, value: V)
    where
        V: UniformValue,
    {
        ctx.gl.set_uniform(&self.handle, name, value);
    }
}
