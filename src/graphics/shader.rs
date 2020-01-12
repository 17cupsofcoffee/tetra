//! Functions and types relating to shader programs.

use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::fs;
use crate::platform::{GraphicsDevice, RawProgram};
use crate::Context;

#[doc(inline)]
pub use crate::platform::UniformValue;

/// The default vertex shader.
///
/// The source code for this shader is available in [`src/resources/shader.vert`](https://github.com/17cupsofcoffee/tetra/blob/master/src/resources/shader.vert).
pub const DEFAULT_VERTEX_SHADER: &str = include_str!("../resources/shader.vert");

/// The default fragment shader.
///
/// The source code for this shader is available in [`src/resources/shader.vert`](https://github.com/17cupsofcoffee/tetra/blob/master/src/resources/shader.frag).
pub const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../resources/shader.frag");

/// A shader program, consisting of a vertex shader and a fragment shader.
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
///
/// # Data Format
///
/// ## Vertex Shaders
///
/// Vertex shaders take in data via three attributes:
///
/// * `a_position` - A `vec2` representing the position of the vertex in world space.
/// * `a_uv` - A `vec2` representing the texture co-ordinates that are associated with the vertex.
/// * `a_color` - A `vec4` representing a color to multiply the output by.
///
/// Position data should be output as a `vec4` to the built-in `gl_Position` variable.
///
/// ## Fragment Shaders
///
/// Color data should be output as a `vec4` to the first output of the shader. This can be the
/// built-in `gl_FragColor` variable, if you so desire.
///
/// ## Uniforms
///
/// By default, the shader is provided with two uniform variables:
///
/// * `u_projection` - A `mat4` which can be used to translate world space co-ordinates into screen space.
/// * `u_texture` - A `sampler2D` which can be used to access color data from the currently active texture.
///
/// You can also set data into your own uniform variables via the `set_uniform` method.
#[derive(Debug, Clone, PartialEq)]
pub struct Shader {
    pub(crate) handle: Rc<RawProgram>,
}

impl Shader {
    /// Creates a new shader program from the given files.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::FailedToLoadAsset` will be returned if the files could not be loaded.
    /// * `TetraError::InvalidShader` will be returned if the shader could not be compiled.
    pub fn new<P>(ctx: &mut Context, vertex_path: P, fragment_path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::with_device(
            &mut ctx.device,
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
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::FailedToLoadAsset` will be returned if the file could not be loaded.
    /// * `TetraError::InvalidShader` will be returned if the shader could not be compiled.
    pub fn from_vertex_file<P>(ctx: &mut Context, path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::with_device(
            &mut ctx.device,
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
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::FailedToLoadAsset` will be returned if the file could not be loaded.
    /// * `TetraError::InvalidShader` will be returned if the shader could not be compiled.
    pub fn from_fragment_file<P>(ctx: &mut Context, path: P) -> Result<Shader>
    where
        P: AsRef<Path>,
    {
        Shader::with_device(
            &mut ctx.device,
            DEFAULT_VERTEX_SHADER,
            &fs::read_to_string(path)?,
        )
    }

    /// Creates a new shader program from the given strings.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::InvalidShader` will be returned if the shader could not be compiled.
    pub fn from_string(
        ctx: &mut Context,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Shader> {
        Shader::with_device(&mut ctx.device, vertex_shader, fragment_shader)
    }

    /// Creates a new shader program from the given vertex shader string.
    ///
    /// The default fragment shader will be used.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::InvalidShader` will be returned if the shader could not be compiled.
    pub fn from_vertex_string<P>(ctx: &mut Context, shader: &str) -> Result<Shader> {
        Shader::with_device(&mut ctx.device, shader, DEFAULT_FRAGMENT_SHADER)
    }

    /// Creates a new shader program from the given fragment shader string.
    ///
    /// The default vertex shader will be used.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::InvalidShader` will be returned if the shader could not be compiled.
    pub fn from_fragment_string<P>(ctx: &mut Context, shader: &str) -> Result<Shader> {
        Shader::with_device(&mut ctx.device, DEFAULT_VERTEX_SHADER, shader)
    }

    pub(crate) fn with_device(
        device: &mut GraphicsDevice,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<Shader> {
        let handle = device.new_program(vertex_shader, fragment_shader)?;

        Ok(Shader {
            handle: Rc::new(handle),
        })
    }

    /// Sets the value of the specifed uniform parameter.
    pub fn set_uniform<V>(&self, ctx: &mut Context, name: &str, value: V)
    where
        V: UniformValue,
    {
        ctx.device.set_uniform(&self.handle, name, value);
    }
}
