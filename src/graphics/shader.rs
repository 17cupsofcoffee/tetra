//! Functions and types relating to shader programs.

use std::cell::{Cell, RefCell};
use std::path::Path;
use std::rc::Rc;

use hashbrown::HashMap;

use crate::error::Result;
use crate::fs;
use crate::graphics::{Color, Texture};
use crate::math::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};
use crate::platform::{GraphicsDevice, RawShader};
use crate::Context;

/// The default vertex shader.
///
/// The source code for this shader is available in [`src/resources/shader.vert`](https://github.com/17cupsofcoffee/tetra/blob/main/src/resources/shader.vert).
pub const DEFAULT_VERTEX_SHADER: &str = include_str!("../resources/shader.vert");

/// The default fragment shader.
///
/// The source code for this shader is available in [`src/resources/shader.vert`](https://github.com/17cupsofcoffee/tetra/blob/main/src/resources/shader.frag).
pub const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../resources/shader.frag");

#[derive(Debug)]
pub(crate) struct Sampler {
    pub(crate) texture: Texture,
    pub(crate) unit: u32,
}

#[derive(Debug)]
pub(crate) struct ShaderSharedData {
    pub(crate) handle: RawShader,
    pub(crate) samplers: RefCell<HashMap<String, Sampler>>,
    pub(crate) next_unit: Cell<u32>,
}

impl PartialEq for ShaderSharedData {
    fn eq(&self, other: &ShaderSharedData) -> bool {
        self.handle.eq(&other.handle)
    }
}

/// A shader program, consisting of a vertex shader and a fragment shader.
///
/// # Data Format
///
/// Shaders are written using [GLSL](https://en.wikipedia.org/wiki/OpenGL_Shading_Language).
///
/// ## Vertex Shaders
///
/// Vertex shaders take in data via three attributes:
///
/// * `a_position` - A `vec2` representing the position of the vertex in world space.
/// * `a_uv` - A `vec2` representing the texture co-ordinates that are associated with the vertex.
/// * `a_color` - A `vec4` representing the color of the vertex. This will be multiplied by
///   `u_diffuse` and the color sampled from `u_texture` (see 'Uniforms' below).
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
/// By default, the shader is provided with three uniform variables:
///
/// * `u_projection` - A `mat4` which can be used to translate world space co-ordinates into screen space.
/// * `u_texture` - A `sampler2D` which can be used to access color data from the currently active texture.
/// * `u_diffuse` - A `vec4` representing the color of the current geometry. This is currently only used to
///   pass through the [`DrawParams::color`](super::DrawParams::color) for a [`Mesh`](super::mesh::Mesh), and will
///   otherwise be set to [`Color::WHITE`].
///
/// You can also set data into your own uniform variables via the [`set_uniform`](Shader::set_uniform) method.
///
/// # Performance
///
/// Creating a `Shader` is a relatively expensive operation. If you can, store them in your
/// [`State`](crate::State) struct rather than recreating them each frame.
///
/// Cloning a `Shader` is a very cheap operation, as the underlying data is shared between the
/// original instance and the clone via [reference-counting](https://doc.rust-lang.org/std/rc/struct.Rc.html).
/// This does mean, however, that updating a `Shader` (for example, setting a uniform) will also
/// update any other clones of that `Shader`.
///
/// # Examples
///
/// The [`shaders`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/shaders.rs)
/// example demonstrates how to draw using a custom shader, supplying inputs via uniform
/// variables.
#[derive(Debug, Clone, PartialEq)]
pub struct Shader {
    pub(crate) data: Rc<ShaderSharedData>,
}

impl Shader {
    /// Creates a new shader program from the given files.
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the
    /// underlying graphics API encounters an error.
    /// * [`TetraError::FailedToLoadAsset`](crate::TetraError::FailedToLoadAsset) will be returned
    /// if the files could not be loaded.
    /// * [`TetraError::InvalidShader`](crate::TetraError::InvalidShader) will be returned if the
    /// shader could not be compiled.
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
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the
    /// underlying graphics API encounters an error.
    /// * [`TetraError::FailedToLoadAsset`](crate::TetraError::FailedToLoadAsset) will be returned
    /// if the file could not be loaded.
    /// * [`TetraError::InvalidShader`](crate::TetraError::InvalidShader) will be returned if the
    /// shader could not be compiled.
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
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the
    /// underlying graphics API encounters an error.
    /// * [`TetraError::FailedToLoadAsset`](crate::TetraError::FailedToLoadAsset) will be returned
    /// if the file could not be loaded.
    /// * [`TetraError::InvalidShader`](crate::TetraError::InvalidShader) will be returned if the
    /// shader could not be compiled.
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
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the
    /// underlying graphics API encounters an error.
    /// * [`TetraError::InvalidShader`](crate::TetraError::InvalidShader) will be returned if the
    /// shader could not be compiled.
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
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the
    /// underlying graphics API encounters an error.
    /// * [`TetraError::InvalidShader`](crate::TetraError::InvalidShader) will be returned if the
    /// shader could not be compiled.
    pub fn from_vertex_string<P>(ctx: &mut Context, shader: &str) -> Result<Shader> {
        Shader::with_device(&mut ctx.device, shader, DEFAULT_FRAGMENT_SHADER)
    }

    /// Creates a new shader program from the given fragment shader string.
    ///
    /// The default vertex shader will be used.
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the
    /// underlying graphics API encounters an error.
    /// * [`TetraError::InvalidShader`](crate::TetraError::InvalidShader) will be returned if the
    /// shader could not be compiled.
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
            data: Rc::new(ShaderSharedData {
                handle,
                samplers: RefCell::new(HashMap::new()),
                next_unit: Cell::new(1),
            }),
        })
    }

    /// Sets the value of the specifed uniform parameter.
    ///
    /// See the [`UniformValue`] trait's docs for a list of which types can be used as a uniform,
    /// and what their corresponding GLSL types are.
    pub fn set_uniform<V>(&self, ctx: &mut Context, name: &str, value: V)
    where
        V: UniformValue,
    {
        value.set_uniform(ctx, self, name)
    }

    pub(crate) fn set_default_uniforms(
        &self,
        device: &mut GraphicsDevice,
        projection: Mat4<f32>,
        diffuse: Color,
    ) -> Result {
        let samplers = self.data.samplers.borrow();

        for sampler in samplers.values() {
            device.attach_texture_to_sampler(&sampler.texture.data.handle, sampler.unit)?;
        }

        let projection_location = device.get_uniform_location(&self.data.handle, "u_projection");

        device.set_uniform_mat4(&self.data.handle, projection_location.as_ref(), projection);

        let diffuse_location = device.get_uniform_location(&self.data.handle, "u_diffuse");

        device.set_uniform_vec4(&self.data.handle, diffuse_location.as_ref(), diffuse.into());

        Ok(())
    }
}

/// Implemented for types that can be passed as a uniform value to a shader.
///
/// As the implementation of this trait currently interacts directly with the platform layer,
/// it cannot be implemented outside of Tetra itself. This may change in the future!
pub trait UniformValue {
    #[doc(hidden)]
    fn set_uniform(&self, ctx: &mut Context, shader: &Shader, name: &str);
}

macro_rules! simple_uniforms {
    ($($t:ty => $f:ident $doc:expr),* $(,)?) => {
        $(
            #[doc = $doc]
            impl UniformValue for $t {
                #[doc(hidden)]
                 fn set_uniform(
                    &self,
                    ctx: &mut Context,
                    shader: &Shader,
                    name: &str,
                ) {
                    let location = ctx.device.get_uniform_location(&shader.data.handle, name);
                    ctx.device.$f(&shader.data.handle, location.as_ref(), *self);
                }
            }
        )*
    };
}

simple_uniforms! {
    i32 => set_uniform_i32 "Can be accessed as an `int` in your shader.",
    u32 => set_uniform_u32 "Can be accessed as a `uint` in your shader.",
    f32 => set_uniform_f32 "Can be accessed as a `float` in your shader.",
    Vec2<f32> => set_uniform_vec2 "Can be accessed as a `vec2` in your shader.",
    Vec3<f32> => set_uniform_vec3 "Can be accessed as a `vec3` in your shader.",
    Vec4<f32> => set_uniform_vec4 "Can be accessed as a `vec4` in your shader.",
    Mat2<f32> => set_uniform_mat2 "Can be accessed as a `mat2` in your shader.",
    Mat3<f32> => set_uniform_mat3 "Can be accessed as a `mat3` in your shader.",
    Mat4<f32> => set_uniform_mat4 "Can be accessed as a `mat4` in your shader.",
}

/// Can be accessed as a `vec4` in your shader.
impl UniformValue for Color {
    #[doc(hidden)]
    fn set_uniform(&self, ctx: &mut Context, shader: &Shader, name: &str) {
        let vec4: Vec4<f32> = (*self).into();
        vec4.set_uniform(ctx, shader, name);
    }
}

/// Can be accessed via a `sampler2D` in your shader.
impl UniformValue for Texture {
    #[doc(hidden)]
    fn set_uniform(&self, ctx: &mut Context, shader: &Shader, name: &str) {
        let mut samplers = shader.data.samplers.borrow_mut();

        if let Some(sampler) = samplers.get_mut(name) {
            if sampler.texture != *self {
                sampler.texture = self.clone();
            }
        } else {
            let next_unit = shader.data.next_unit.get();

            samplers.insert(
                name.to_owned(),
                Sampler {
                    texture: self.clone(),
                    unit: next_unit,
                },
            );

            // Sampler uniforms have to be set via glUniform1i
            (next_unit as i32).set_uniform(ctx, shader, name);

            shader.data.next_unit.set(next_unit + 1);
        }
    }
}

/// Any type that can be passed by value to a shader can also be passed by reference.
impl<'a, T> UniformValue for &'a T
where
    T: UniformValue,
{
    #[doc(hidden)]
    fn set_uniform(&self, ctx: &mut Context, shader: &Shader, name: &str) {
        {
            (**self).set_uniform(ctx, shader, name);
        }
    }
}
