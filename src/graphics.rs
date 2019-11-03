//! Functions and types relating to rendering.
//!
//! This module implements a (hopefully!) efficent quad renderer, which will queue up
//! drawing operations until it is absolutely necessary to send them to the graphics
//! hardware. This allows us to minimize the number of draw calls made, speeding up
//! rendering.

pub mod animation;
mod buffers;
mod canvas;
mod color;
mod drawable;
pub(crate) mod opengl;
mod rectangle;
pub mod scaling;
mod shader;
mod text;
mod texture;
pub mod ui;

pub(crate) use buffers::*;
pub use canvas::*;
pub use color::*;
pub use drawable::*;
pub use rectangle::*;
pub use shader::*;
pub use text::*;
pub use texture::*;

use glyph_brush::{GlyphBrush, GlyphBrushBuilder};

use crate::error::Result;
use crate::graphics::opengl::{BufferUsage, FrontFace, GLDevice};
use crate::graphics::text::FontQuad;
use crate::math::{self, Mat4, Vec2};
use crate::platform;
use crate::window;
use crate::Context;

const MAX_SPRITES: usize = 2048;
const MAX_VERTICES: usize = MAX_SPRITES * 4; // Cannot be greater than 32767!
const MAX_INDICES: usize = MAX_SPRITES * 6;
const VERTEX_STRIDE: usize = 8;
const INDEX_ARRAY: [u32; 6] = [0, 1, 2, 2, 3, 0];
const DEFAULT_FONT: &[u8] = include_bytes!("./resources/DejaVuSansMono.ttf");

#[derive(PartialEq)]
pub(crate) enum ActiveTexture {
    FontCache,
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
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    texture: ActiveTexture,
    font_cache_texture: Texture,

    shader: ActiveShader,
    default_shader: Shader,

    canvas: ActiveCanvas,

    window_projection: Mat4,

    vertex_data: Vec<f32>,
    element_capacity: i32,
    element_count: i32,

    font_cache: GlyphBrush<'static, FontQuad>,
}

impl GraphicsContext {
    pub(crate) fn new(
        device: &mut GLDevice,
        window_width: i32,
        window_height: i32,
    ) -> Result<GraphicsContext> {
        let indices: Vec<u32> = INDEX_ARRAY
            .iter()
            .cycle()
            .take(MAX_INDICES)
            .enumerate()
            .map(|(i, vertex)| vertex + i as u32 / 6 * 4)
            .collect();

        let vertex_buffer = device.new_vertex_buffer(
            MAX_VERTICES * VERTEX_STRIDE,
            VERTEX_STRIDE,
            BufferUsage::DynamicDraw,
        )?;

        device.set_vertex_buffer_attribute(&vertex_buffer, 0, 2, 0);
        device.set_vertex_buffer_attribute(&vertex_buffer, 1, 2, 2);
        device.set_vertex_buffer_attribute(&vertex_buffer, 2, 4, 4);

        let index_buffer = device.new_index_buffer(MAX_INDICES, BufferUsage::StaticDraw)?;

        device.set_index_buffer_data(&index_buffer, &indices, 0);

        let default_shader = device.new_shader(
            shader::DEFAULT_VERTEX_SHADER,
            shader::DEFAULT_FRAGMENT_SHADER,
        )?;

        let font_cache = GlyphBrushBuilder::using_font_bytes(DEFAULT_FONT).build();
        let (width, height) = font_cache.texture_dimensions();
        let font_cache_texture = device.new_texture_empty(width as i32, height as i32)?;

        Ok(GraphicsContext {
            vertex_buffer,
            index_buffer,

            texture: ActiveTexture::FontCache,
            font_cache_texture,

            shader: ActiveShader::Default,
            default_shader,

            canvas: ActiveCanvas::Window,

            window_projection: math::ortho(
                0.0,
                window_width as f32,
                window_height as f32,
                0.0,
                -1.0,
                1.0,
            ),

            vertex_data: Vec::with_capacity(MAX_VERTICES * VERTEX_STRIDE),
            element_capacity: MAX_INDICES as i32,
            element_count: 0,

            font_cache,
        })
    }
}

/// Clears the screen (or a canvas, if one is enabled) to the specified color.
pub fn clear(ctx: &mut Context, color: Color) {
    ctx.gl.clear(color.r, color.g, color.b, color.a);
}

// TODO: This function really needs cleaning up before it can be exposed publicly.

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

    if ctx.graphics.element_count >= ctx.graphics.element_capacity {
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
        // 1
        ox1,
        oy1,
        u1,
        v1,
        params.color.r,
        params.color.g,
        params.color.b,
        params.color.a,
        // 2
        ox2,
        oy2,
        u1,
        v2,
        params.color.r,
        params.color.g,
        params.color.b,
        params.color.a,
        // 3
        ox3,
        oy3,
        u2,
        v2,
        params.color.r,
        params.color.g,
        params.color.b,
        params.color.a,
        // 4
        ox4,
        oy4,
        u2,
        v1,
        params.color.r,
        params.color.g,
        params.color.b,
        params.color.a,
    ]);

    ctx.graphics.element_count += 6;
}

/// Draws an object to the screen (or to a canvas, if one is enabled).
///
/// This function simply calls [`draw`](trait.Drawable.html#tymethod.draw) on the passed object - it is
/// provided to allow you to avoid having to import the [`Drawable`](trait.Drawable.html) trait as well
/// as the `graphics` module.
pub fn draw<D: Drawable, P: Into<DrawParams>>(ctx: &mut Context, drawable: &D, params: P) {
    drawable.draw(ctx, params);
}

/// Sets the texture that is currently being used for rendering.
///
/// If the texture is different from the one that is currently in use, this will trigger a
/// [`flush`](fn.flush.html) to the graphics hardware - try to avoid texture swapping as
/// much as you can.
pub fn set_texture(ctx: &mut Context, texture: &Texture) {
    set_texture_ex(ctx, ActiveTexture::User(texture.clone()));
}

pub(crate) fn set_texture_ex(ctx: &mut Context, texture: ActiveTexture) {
    if texture != ctx.graphics.texture {
        flush(ctx);
        ctx.graphics.texture = texture;
    }
}

/// Sets the shader that is currently being used for rendering.
///
/// If the shader is different from the one that is currently in use, this will trigger a
/// [`flush`](fn.flush.html) to the graphics hardware - try to avoid shader swapping as
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
/// [`flush`](fn.flush.html) to the graphics hardware.
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
        ctx.graphics.canvas = canvas;

        match &ctx.graphics.canvas {
            ActiveCanvas::Window => {
                ctx.gl.bind_canvas(None);
                ctx.gl.front_face(FrontFace::CounterClockwise);
                ctx.gl
                    .viewport(0, 0, window::get_width(ctx), window::get_height(ctx));
            }
            ActiveCanvas::User(r) => {
                ctx.gl.bind_canvas(Some(r));
                ctx.gl.front_face(FrontFace::Clockwise);
                ctx.gl.viewport(0, 0, r.width(), r.height());
            }
        }
    }
}

/// Sends queued data to the graphics hardware.
///
/// You usually will not have to call this manually, as [`set_texture`](fn.set_texture.html) and
/// [`present`](fn.present.html) will automatically flush when necessary. Try to keep flushing
/// to a minimum, as this will reduce the number of draw calls made to the graphics device.
pub fn flush(ctx: &mut Context) {
    if !ctx.graphics.vertex_data.is_empty() {
        let texture = match &ctx.graphics.texture {
            ActiveTexture::FontCache => &ctx.graphics.font_cache_texture,
            ActiveTexture::User(t) => t,
        };

        let shader = match &ctx.graphics.shader {
            ActiveShader::Default => &ctx.graphics.default_shader,
            ActiveShader::User(s) => s,
        };

        let projection = match &ctx.graphics.canvas {
            ActiveCanvas::Window => &ctx.graphics.window_projection,
            ActiveCanvas::User(r) => &r.projection,
        };

        ctx.gl.set_uniform(shader, "u_projection", &projection);

        ctx.gl
            .set_vertex_buffer_data(&ctx.graphics.vertex_buffer, &ctx.graphics.vertex_data, 0);

        ctx.gl.draw_elements(
            &ctx.graphics.vertex_buffer,
            &ctx.graphics.index_buffer,
            texture,
            shader,
            ctx.graphics.element_count,
        );

        ctx.graphics.vertex_data.clear();
        ctx.graphics.element_count = 0;
    }
}

/// Presents the result of drawing commands to the screen, scaling/letterboxing if necessary.
///
/// If any custom shaders/canvases are set, this function will unset them -
/// don't rely on the state of one render carrying over to the next!
///
/// You usually will not have to call this manually, as it is called for you at the end of every
/// frame. Note that calling it will trigger a [`flush`](fn.flush.html) to the graphics hardware.
pub fn present(ctx: &mut Context) {
    flush(ctx);

    platform::swap_buffers(ctx);
}

/// Returns the filter mode that will be used by newly created textures and canvases.
pub fn get_default_filter_mode(ctx: &Context) -> FilterMode {
    ctx.gl.get_default_filter_mode()
}

/// Sets the filter mode that will be used by newly created textures and canvases.
pub fn set_default_filter_mode(ctx: &mut Context, filter_mode: FilterMode) {
    ctx.gl.set_default_filter_mode(filter_mode);
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
    GraphicsDeviceInfo {
        vendor: ctx.gl.get_vendor(),
        renderer: ctx.gl.get_renderer(),
        opengl_version: ctx.gl.get_version(),
        glsl_version: ctx.gl.get_shading_language_version(),
    }
}

pub(crate) fn set_window_projection(ctx: &mut Context, width: i32, height: i32) {
    ctx.graphics.window_projection = math::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

    if let ActiveCanvas::Window = ctx.graphics.canvas {
        ctx.gl.viewport(0, 0, width, height);
    }
}
