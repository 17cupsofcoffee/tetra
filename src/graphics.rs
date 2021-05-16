//! Functions and types relating to rendering.
//!
//! This module implements a (hopefully!) efficent quad renderer, which will queue up
//! drawing operations until it is absolutely necessary to send them to the graphics
//! hardware. This allows us to minimize the number of draw calls made, speeding up
//! rendering.

pub mod animation;
mod camera;
mod canvas;
mod color;
mod drawparams;
pub mod mesh;
mod rectangle;
pub mod scaling;
mod shader;
pub mod text;
mod texture;

pub use camera::*;
pub use canvas::*;
pub use color::*;
pub use drawparams::*;
pub use rectangle::*;
pub use shader::*;
pub use texture::*;

use crate::error::Result;
use crate::math::{FrustumPlanes, Mat4, Vec2};
use crate::platform::{GraphicsDevice, RawIndexBuffer, RawVertexBuffer};
use crate::window;
use crate::Context;

use self::mesh::{BufferUsage, Vertex, VertexWinding};

const MAX_SPRITES: usize = 2048;
const MAX_VERTICES: usize = MAX_SPRITES * 4; // Cannot be greater than 32767!
const MAX_INDICES: usize = MAX_SPRITES * 6;
const INDEX_ARRAY: [u32; 6] = [0, 1, 2, 2, 3, 0];

#[derive(PartialEq)]
pub(crate) enum ActiveTexture {
    Default,
    User(Texture),
}

#[derive(PartialEq)]
pub(crate) enum ActiveShader {
    Default,
    User(Shader),
}

#[derive(PartialEq)]
pub(crate) enum ActiveCanvas {
    Window,
    User(Canvas),
}

pub(crate) struct GraphicsContext {
    vertex_buffer: RawVertexBuffer,
    index_buffer: RawIndexBuffer,

    texture: ActiveTexture,
    default_texture: Texture,
    default_filter_mode: FilterMode,

    shader: ActiveShader,
    default_shader: Shader,

    canvas: ActiveCanvas,

    projection_matrix: Mat4<f32>,
    transform_matrix: Mat4<f32>,

    vertex_data: Vec<Vertex>,
    element_count: usize,

    blend_mode: BlendMode,
}

impl GraphicsContext {
    pub(crate) fn new(
        device: &mut GraphicsDevice,
        window_width: i32,
        window_height: i32,
    ) -> Result<GraphicsContext> {
        let vertex_buffer = device.new_vertex_buffer(MAX_VERTICES, BufferUsage::Dynamic)?;
        let index_buffer = device.new_index_buffer(MAX_INDICES, BufferUsage::Static)?;

        let indices: Vec<u32> = INDEX_ARRAY
            .iter()
            .cycle()
            .take(MAX_INDICES)
            .enumerate()
            .map(|(i, vertex)| vertex + i as u32 / 6 * 4)
            .collect();

        device.set_index_buffer_data(&index_buffer, &indices, 0);

        let default_texture =
            Texture::with_device(device, 1, 1, &[255, 255, 255, 255], FilterMode::Nearest)?;

        let default_filter_mode = FilterMode::Nearest;

        let default_shader = Shader::with_device(
            device,
            shader::DEFAULT_VERTEX_SHADER,
            shader::DEFAULT_FRAGMENT_SHADER,
        )?;

        Ok(GraphicsContext {
            vertex_buffer,
            index_buffer,

            texture: ActiveTexture::Default,
            default_texture,
            default_filter_mode,

            shader: ActiveShader::Default,
            default_shader,

            canvas: ActiveCanvas::Window,

            projection_matrix: ortho(window_width as f32, window_height as f32, false),
            transform_matrix: Mat4::identity(),

            vertex_data: Vec::with_capacity(MAX_VERTICES),
            element_count: 0,

            blend_mode: BlendMode::default(),
        })
    }
}

/// Clears the screen (or a canvas, if one is enabled) to the specified color.
pub fn clear(ctx: &mut Context, color: Color) {
    ctx.device.clear(color);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn push_quad(
    ctx: &mut Context,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    mut u1: f32,
    mut v1: f32,
    mut u2: f32,
    mut v2: f32,
    params: &DrawParams,
) {
    // This function is a bit hairy, but it's more performant than doing the matrix math every
    // frame by a *lot* (at least going by the BunnyMark example). The logic is roughly based
    // on how FNA and LibGDX implement their spritebatches.
    //
    // TODO: This function really needs cleaning up before it can be exposed publicly.

    if ctx.graphics.element_count + 6 > MAX_INDICES {
        flush(ctx);
    }

    let mut fx = (x1 - params.origin.x) * params.scale.x;
    let mut fy = (y1 - params.origin.y) * params.scale.y;
    let mut fx2 = (x2 - params.origin.x) * params.scale.x;
    let mut fy2 = (y2 - params.origin.y) * params.scale.y;

    if fx2 < fx {
        std::mem::swap(&mut fx, &mut fx2);
        std::mem::swap(&mut u1, &mut u2);
    }

    if fy2 < fy {
        std::mem::swap(&mut fy, &mut fy2);
        std::mem::swap(&mut v1, &mut v2);
    }

    // Branching here might be a bit of a premature optimization...
    let (ox1, oy1, ox2, oy2, ox3, oy3, ox4, oy4) = if params.rotation == 0.0 {
        (
            params.position.x + fx,
            params.position.y + fy,
            params.position.x + fx,
            params.position.y + fy2,
            params.position.x + fx2,
            params.position.y + fy2,
            params.position.x + fx2,
            params.position.y + fy,
        )
    } else {
        let sin = params.rotation.sin();
        let cos = params.rotation.cos();
        (
            params.position.x + (cos * fx) - (sin * fy),
            params.position.y + (sin * fx) + (cos * fy),
            params.position.x + (cos * fx) - (sin * fy2),
            params.position.y + (sin * fx) + (cos * fy2),
            params.position.x + (cos * fx2) - (sin * fy2),
            params.position.y + (sin * fx2) + (cos * fy2),
            params.position.x + (cos * fx2) - (sin * fy),
            params.position.y + (sin * fx2) + (cos * fy),
        )
    };

    ctx.graphics.vertex_data.extend_from_slice(&[
        Vertex::new(Vec2::new(ox1, oy1), Vec2::new(u1, v1), params.color),
        Vertex::new(Vec2::new(ox2, oy2), Vec2::new(u1, v2), params.color),
        Vertex::new(Vec2::new(ox3, oy3), Vec2::new(u2, v2), params.color),
        Vertex::new(Vec2::new(ox4, oy4), Vec2::new(u2, v1), params.color),
    ]);

    ctx.graphics.element_count += 6;
}

pub(crate) fn set_texture(ctx: &mut Context, texture: &Texture) {
    set_texture_ex(ctx, ActiveTexture::User(texture.clone()));
}

pub(crate) fn set_texture_ex(ctx: &mut Context, texture: ActiveTexture) {
    if texture != ctx.graphics.texture {
        flush(ctx);
        ctx.graphics.texture = texture;
    }
}

/// Sets the blend mode used for future drawing operations.
///
/// The blend mode will be used to determine how drawn content will be blended
/// with the screen (or with a [`Canvas`], if one is active).
pub fn set_blend_mode(ctx: &mut Context, blend_mode: BlendMode) {
    if blend_mode != ctx.graphics.blend_mode {
        flush(ctx);
        ctx.graphics.blend_mode = blend_mode;
    }
    ctx.device.set_blend_mode(blend_mode);
}

/// Resets the blend mode to the default.
pub fn reset_blend_mode(ctx: &mut Context) {
    set_blend_mode(ctx, Default::default());
}

/// Sets the shader that is currently being used for rendering.
///
/// If the shader is different from the one that is currently in use, this will trigger a
/// [`flush`] to the graphics hardware - try to avoid shader swapping as
/// much as you can.
pub fn set_shader(ctx: &mut Context, shader: &Shader) {
    set_shader_ex(ctx, ActiveShader::User(shader.clone()));
}

/// Sets the renderer back to using the default shader.
pub fn reset_shader(ctx: &mut Context) {
    set_shader_ex(ctx, ActiveShader::Default);
}

pub(crate) fn set_shader_ex(ctx: &mut Context, shader: ActiveShader) {
    if shader != ctx.graphics.shader {
        flush(ctx);
        ctx.graphics.shader = shader;
    }
}

/// Sets the renderer to redirect all drawing commands to the specified canvas.
///
/// If the canvas is different from the one that is currently in use, this will trigger a
/// [`flush`] to the graphics hardware.
pub fn set_canvas(ctx: &mut Context, canvas: &Canvas) {
    set_canvas_ex(ctx, ActiveCanvas::User(canvas.clone()));
}

/// Sets the renderer back to drawing to the screen directly.
pub fn reset_canvas(ctx: &mut Context) {
    set_canvas_ex(ctx, ActiveCanvas::Window);
}

pub(crate) fn set_canvas_ex(ctx: &mut Context, canvas: ActiveCanvas) {
    if canvas != ctx.graphics.canvas {
        flush(ctx);
        resolve_canvas(ctx);

        ctx.graphics.canvas = canvas;

        match &ctx.graphics.canvas {
            ActiveCanvas::Window => {
                let (width, height) = window::get_size(ctx);
                let (physical_width, physical_height) = window::get_physical_size(ctx);

                ctx.graphics.projection_matrix = ortho(width as f32, height as f32, false);
                ctx.device.viewport(0, 0, physical_width, physical_height);

                ctx.device.set_canvas(None);
            }
            ActiveCanvas::User(r) => {
                let (width, height) = r.size();

                ctx.graphics.projection_matrix = ortho(width as f32, height as f32, true);
                ctx.device.viewport(0, 0, width, height);

                ctx.device.set_canvas(Some(&r.handle));
            }
        }
    }
}

fn resolve_canvas(ctx: &mut Context) {
    if let ActiveCanvas::User(c) = &ctx.graphics.canvas {
        ctx.device.resolve(&c.handle, &c.texture.data.handle);
    }
}

/// Sends queued data to the graphics hardware.
///
/// You usually will not have to call this manually, as the graphics API will
/// automatically flush when necessary. Try to keep flushing to a minimum,
/// as this will reduce the number of draw calls made to the
/// graphics device.
pub fn flush(ctx: &mut Context) {
    if !ctx.graphics.vertex_data.is_empty() {
        let texture = match &ctx.graphics.texture {
            ActiveTexture::Default => return,
            ActiveTexture::User(t) => t,
        };

        let shader = match &ctx.graphics.shader {
            ActiveShader::Default => &ctx.graphics.default_shader,
            ActiveShader::User(s) => s,
        };

        // TODO: Failing to apply the defaults should be handled more gracefully than this,
        // but we can't do that without breaking changes.
        let _ = shader.set_default_uniforms(
            &mut ctx.device,
            ctx.graphics.projection_matrix * ctx.graphics.transform_matrix,
            Color::WHITE,
        );

        ctx.device.cull_face(true);

        // Because canvas rendering is effectively done upside-down, the winding order is the opposite
        // of what you'd expect in that case.
        ctx.device.front_face(match &ctx.graphics.canvas {
            ActiveCanvas::Window => VertexWinding::CounterClockwise,
            ActiveCanvas::User(_) => VertexWinding::Clockwise,
        });

        ctx.device.set_vertex_buffer_data(
            &ctx.graphics.vertex_buffer,
            &ctx.graphics.vertex_data,
            0,
        );

        ctx.device.draw(
            &ctx.graphics.vertex_buffer,
            Some(&ctx.graphics.index_buffer),
            &texture.data.handle,
            &shader.data.handle,
            0,
            ctx.graphics.element_count,
        );

        ctx.graphics.vertex_data.clear();
        ctx.graphics.element_count = 0;
    }
}

/// Presents the result of drawing commands to the screen.
///
/// If any custom shaders/canvases are set, this function will unset them -
/// don't rely on the state of one render carrying over to the next!
///
/// You usually will not have to call this manually, as it is called for you at the end of every
/// frame. Note that calling it will trigger a [`flush`] to the graphics hardware.
pub fn present(ctx: &mut Context) {
    flush(ctx);

    ctx.window.swap_buffers();
}

/// Returns the filter mode that will be used by newly created textures and canvases.
pub fn get_default_filter_mode(ctx: &Context) -> FilterMode {
    ctx.graphics.default_filter_mode
}

/// Sets the filter mode that will be used by newly created textures and canvases.
pub fn set_default_filter_mode(ctx: &mut Context, filter_mode: FilterMode) {
    ctx.graphics.default_filter_mode = filter_mode;
}

/// Information about the device currently being used to render graphics.
#[derive(Debug, Clone)]
pub struct GraphicsDeviceInfo {
    /// The name of the company responsible for the OpenGL implementation.
    pub vendor: String,

    /// The name of the renderer. This usually corresponds to the name
    /// of the physical device.
    pub renderer: String,

    /// The version of OpenGL that is being used.
    pub opengl_version: String,

    /// The version of GLSL that is being used.
    pub glsl_version: String,
}

/// Retrieves information about the device currently being used to render graphics.
///
/// This may be useful for debugging/logging purposes.
pub fn get_device_info(ctx: &Context) -> GraphicsDeviceInfo {
    ctx.device.get_info()
}

/// Returns the current transform matrix.
pub fn get_transform_matrix(ctx: &Context) -> Mat4<f32> {
    ctx.graphics.transform_matrix
}

/// Sets the transform matrix.
///
/// This can be used to apply global transformations to subsequent draw calls.
pub fn set_transform_matrix(ctx: &mut Context, matrix: Mat4<f32>) {
    flush(ctx);

    ctx.graphics.transform_matrix = matrix;
}

/// Resets the transform matrix.
///
/// This is a shortcut for calling [`graphics::set_transform_matrix(ctx, Mat4::identity())`](set_transform_matrix).
pub fn reset_transform_matrix(ctx: &mut Context) {
    set_transform_matrix(ctx, Mat4::identity());
}

/// Sets the scissor rectangle.
///
/// While the scissor is enabled, any rendering that falls outside the specified rectangle of
/// the screen (or the current canvas, if one is active) will be be ignored. This includes
/// calls to [`clear`]. This can be useful for things like UI rendering.
///
/// To disable the scissor, call [`reset_scissor`].
///
/// Note that the position/size of the scissor rectangle is not affected by the transform
/// matrix - it always operates in screen/canvas co-ordinates.
pub fn set_scissor(ctx: &mut Context, scissor_rect: Rectangle<i32>) {
    flush(ctx);

    match &ctx.graphics.canvas {
        ActiveCanvas::Window => {
            let physical_height = window::get_physical_height(ctx);

            // OpenGL uses bottom-left co-ordinates, while Tetra uses
            // top-left co-ordinates - to present a consistent API, we
            // flip the Y component here.
            ctx.device.scissor(
                scissor_rect.x,
                physical_height - (scissor_rect.y + scissor_rect.height),
                scissor_rect.width,
                scissor_rect.height,
            );
        }

        ActiveCanvas::User(_) => {
            // Canvas rendering is effectively done upside-down, so we don't
            // need to flip the co-ordinates here.
            ctx.device.scissor(
                scissor_rect.x,
                scissor_rect.y,
                scissor_rect.width,
                scissor_rect.height,
            );
        }
    }

    ctx.device.scissor_test(true);
}

/// Disables the scissor rectangle.
pub fn reset_scissor(ctx: &mut Context) {
    flush(ctx);

    ctx.device.scissor_test(false);
}

/// Sets the global stencil behavior.
///
/// The stencil buffer is an invisible drawing target that you can
/// use as a mask for other drawing operations. For example, you
/// might want to crop an image to a circle. You can do this by
/// drawing a circle to the stencil buffer, then using that buffer
/// as a mask while drawing the image.
///
/// In order to use stencils, you must be rendering to a target that was
/// created with a stencil buffer attached. To enable this for the main
/// backbuffer, set [`ContextBuilder::stencil_buffer`](crate::ContextBuilder::stencil_buffer)
/// to `true` when creating your context. To enable this for a canvas,
/// initialize it via [`Canvas::builder`], with [`stencil_buffer`](CanvasBuilder::stencil_buffer)
/// set to true.
pub fn set_stencil_state(ctx: &mut Context, state: StencilState) {
    flush(ctx);
    ctx.device.set_stencil_state(state);
}

/// Clears the stencil buffer to the specified value.
pub fn clear_stencil(ctx: &mut Context, value: u8) {
    flush(ctx);
    ctx.device.clear_stencil(value);
}

/// Sets which color components are drawn to the screen.
///
/// This is useful in conjunction with [`set_stencil_state`]
/// to draw to the stencil buffer without also drawing to the
/// visible pixels on screen.
pub fn set_color_mask(ctx: &mut Context, red: bool, green: bool, blue: bool, alpha: bool) {
    flush(ctx);
    ctx.device.set_color_mask(red, green, blue, alpha);
}

pub(crate) fn set_viewport_size(ctx: &mut Context) {
    if let ActiveCanvas::Window = ctx.graphics.canvas {
        let (width, height) = window::get_size(ctx);
        let (physical_width, physical_height) = window::get_physical_size(ctx);

        ctx.graphics.projection_matrix = ortho(width as f32, height as f32, false);
        ctx.device.viewport(0, 0, physical_width, physical_height);
    }
}

pub(crate) fn ortho(width: f32, height: f32, flipped: bool) -> Mat4<f32> {
    Mat4::orthographic_rh_no(FrustumPlanes {
        left: 0.0,
        right: width,
        bottom: if flipped { 0.0 } else { height },
        top: if flipped { height } else { 0.0 },
        near: -1.0,
        far: 1.0,
    })
}

// blend mode logic is taken from LÖVE:
// https://github.com/love2d/love/blob/master/src/modules/graphics/opengl/Graphics.cpp#L1234

/// The different ways of blending colors.
///
/// The active blend mode will be used to determine how drawn content will be blended
/// with the screen (or with a [`Canvas`], if one is active).
///
/// For modes where the alpha component of the color can affect the output, an
/// additional [`BlendAlphaMode`] parameter is provided, which determines if
/// the colour should be multiplied by its alpha before blending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    /// The alpha of the drawn content will determine its opacity.
    ///
    /// This is the default behaviour.
    Alpha(BlendAlphaMode),

    /// The pixel colors of the drawn content will be added to the pixel colors
    /// already in the target. The target's alpha will not be affected.
    Add(BlendAlphaMode),

    /// The pixel colors of the drawn content will be subtracted from the pixel colors
    /// already in the target. The target's alpha will not be affected.
    Subtract(BlendAlphaMode),

    /// The pixel colors of the drawn content will be multiplied with the pixel colors
    /// already in the target. The alpha component will also be multiplied.
    Multiply,
}

impl Default for BlendMode {
    fn default() -> BlendMode {
        BlendMode::Alpha(BlendAlphaMode::Multiply)
    }
}

/// How to treat alpha values when blending colors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendAlphaMode {
    /// The RGB components of the color are multiplied by the alpha component before
    /// blending with the target.
    ///
    /// This is the default behaviour.
    Multiply,

    /// The RGB components of the color are *not* multiplied by the alpha component before
    /// blending with the target.
    ///
    /// For this mode to work correctly, you must have multiplied the RGB components of
    /// the colour by the alpha component at some previous point in time (e.g. in your
    /// code, or in your asset pipeline).
    Premultiplied,
}

impl Default for BlendAlphaMode {
    fn default() -> BlendAlphaMode {
        BlendAlphaMode::Multiply
    }
}

/// The test for whether a pixel is visible when using
/// a stencil.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilTest {
    /// The pixel is never visible.
    Never,

    /// The pixel is visible if the
    /// [reference value](StencilState::reference_value) is
    /// less than the value in the stencil buffer.
    LessThan,

    /// The pixel is visible if the
    /// [reference value](StencilState::reference_value) is
    /// less than or equal to the value in the stencil
    /// buffer.
    LessThanOrEqualTo,

    /// The pixel is visible if the
    /// [reference value](StencilState::reference_value) is
    /// equal to the value in the stencil buffer.
    EqualTo,

    /// The pixel is visible if the
    /// [reference value](StencilState::reference_value) is
    /// not equal to the value in the stencil buffer.
    NotEqualTo,

    /// The pixel is visible if the
    /// [reference value](StencilState::reference_value) is
    /// greater than the value in the stencil buffer.
    GreaterThan,

    /// The pixel is visible if the
    /// [reference value](StencilState::reference_value) is
    /// greater than or equal to the value in the stencil
    /// buffer.
    GreaterThanOrEqualTo,

    /// The pixel is always visible.
    Always,
}

/// How drawing operations should modify the stencil buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StencilAction {
    /// Drawing operations will not modify the stencil buffer.
    Keep,

    /// Drawing operations will set the corresponding values
    /// in the stencil buffer to 0.
    Zero,

    /// Drawing operations will replace the corresponding stencil
    /// values with the [reference value](StencilState::reference_value).
    Replace,

    /// Drawing operations will increment the corresponding stencil
    /// values by 1.
    Increment,

    /// Drawing operations will increment the corresponding stencil
    /// values by 1. If a value of 255 is incremented, it will wrap
    /// back around to 0.
    IncrementWrap,

    /// Drawing operations will decrement the corresponding stencil
    /// values by 1.
    Decrement,

    /// Drawing operations will decrement the corresponding stencil
    /// values by 1. If a value of 0 is decremented, it will wrap
    /// back around to 255.
    DecrementWrap,

    /// Drawing operations will bitwise invert the corresponding
    /// stencil values.
    Invert,
}

/// Represents a global stencil configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StencilState {
    /// Whether stencil testing is enabled.
    ///
    /// When set to `true`, pixels drawn will be hidden
    /// or visible depending on the stencil test and the
    /// contents of the stencil buffer.
    pub enabled: bool,

    /// How drawing operations will affect the stencil buffer.
    pub action: StencilAction,

    /// How drawn pixels will be compared to the contents
    /// of the stencil buffer to determine if they're visible.
    pub test: StencilTest,

    /// The value used for most [`StencilTest`]s and
    /// [`StencilAction::Replace`].
    pub reference_value: u8,

    /// A bitmask that will be ANDed with stencil values
    /// before they're written to the buffer.
    pub write_mask: u8,

    /// A bitmask that will be ANDed with both the reference
    /// value and the stencil value before a stencil test
    /// occurs.
    pub read_mask: u8,
}

impl StencilState {
    /// Creates a stencil configuration that will disable use
    /// of the stencil buffer.
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            action: StencilAction::Keep,
            test: StencilTest::Always,
            reference_value: 0,
            write_mask: 0x00,
            read_mask: 0x00,
        }
    }

    /// Creates a stencil configuration that will write pixels
    /// to the stencil buffer.
    pub fn write(action: StencilAction, reference_value: u8) -> Self {
        Self {
            enabled: true,
            action,
            test: StencilTest::Always,
            reference_value,
            write_mask: 0xFF,
            read_mask: 0xFF,
        }
    }

    /// Creates a stencil configuration that will compare drawn
    /// pixels to the contents of the stencil buffer to determine
    /// which pixels are visible.
    pub fn read(test: StencilTest, reference_value: u8) -> Self {
        Self {
            enabled: true,
            action: StencilAction::Keep,
            test,
            reference_value,
            write_mask: 0xFF,
            read_mask: 0xFF,
        }
    }
}
