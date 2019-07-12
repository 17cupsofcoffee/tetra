//! Functions and types relating to shader programs.

use std::fs;
use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::glm::Mat4;
use crate::platform::opengl::GLProgram;
use crate::Context;

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
        ctx.graphics_device.create_shader(
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
        ctx.graphics_device
            .create_shader(&fs::read_to_string(path)?, DEFAULT_FRAGMENT_SHADER)
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
        ctx.graphics_device
            .create_shader(DEFAULT_VERTEX_SHADER, &fs::read_to_string(path)?)
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
        ctx.graphics_device
            .create_shader(vertex_shader, fragment_shader)
    }

    /// Sets the value of the specifed uniform parameter.
    pub fn set_uniform<V>(&mut self, ctx: &mut Context, name: &str, value: V)
    where
        V: UniformValue,
    {
        ctx.graphics_device.set_uniform(self, name, value);
    }
}

/// Represents a type that can be passed as a uniform value to a shader.
///
/// As the implementation of this trait currently interacts directly with the OpenGL layer,
/// it's marked as a [sealed trait](https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed),
/// and can't be implemented outside of Tetra. This might change in the future!
pub trait UniformValue: sealed::UniformValueTypes {
    #[doc(hidden)]
    unsafe fn set_uniform(&self, shader: &Shader, location: Option<u32>);
}

mod sealed {
    use super::*;
    pub trait UniformValueTypes {}
    impl UniformValueTypes for i32 {}
    impl UniformValueTypes for f32 {}
    impl UniformValueTypes for Mat4 {}
    impl<'a, T> UniformValueTypes for &'a T where T: UniformValueTypes {}
}
