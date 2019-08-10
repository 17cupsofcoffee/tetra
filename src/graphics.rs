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
pub(crate) mod opengl;
mod scaling;
mod shader;
mod text;
mod texture;
pub mod ui;

pub use self::canvas::*;
pub use self::color::*;
pub use self::scaling::*;
pub use self::shader::*;
pub use self::text::*;
pub use self::texture::*;
pub use crate::glm::Vec2;
pub(crate) use crate::graphics::buffers::{IndexBuffer, VertexBuffer};

use glyph_brush::{GlyphBrush, GlyphBrushBuilder};

use crate::error::Result;
use crate::glm::{self, Mat4};
use crate::graphics::opengl::{BufferUsage, FrontFace, GLDevice};
use crate::graphics::text::FontQuad;
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
    Backbuffer,
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
    Backbuffer,
    Window,
    User(Canvas),
}

pub(crate) struct GraphicsContext {
    vertex_buffer: VertexBuffer,
    index_buffer: IndexBuffer,

    texture: Option<ActiveTexture>,
    font_cache_texture: Texture,

    shader: ActiveShader,
    default_shader: Shader,

    window_projection: Mat4,

    canvas: ActiveCanvas,
    backbuffer: Canvas,

    vertex_data: Vec<f32>,
    element_capacity: i32,
    element_count: i32,

    internal_width: i32,
    internal_height: i32,
    scaling: ScreenScaling,
    screen_rect: Rectangle,
    letterbox_color: Color,

    font_cache: GlyphBrush<'static, FontQuad>,
}

impl GraphicsContext {
    pub(crate) fn new(
        device: &mut GLDevice,
        window_width: i32,
        window_height: i32,
        internal_width: i32,
        internal_height: i32,
        scaling: ScreenScaling,
    ) -> Result<GraphicsContext> {
        let (backbuffer_width, backbuffer_height) = match scaling {
            ScreenScaling::Resize => (window_width, window_height),
            _ => (internal_width, internal_height),
        };

        let screen_rect =
            scaling.get_screen_rect(internal_width, internal_height, window_width, window_height);

        let backbuffer = device.new_canvas(backbuffer_width, backbuffer_height, false)?;

        device.viewport(0, 0, backbuffer_width, backbuffer_height);
        device.front_face(FrontFace::Clockwise);

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

            texture: None,
            font_cache_texture,

            shader: ActiveShader::Default,
            default_shader,

            window_projection: glm::ortho(
                0.0,
                window_width as f32,
                window_height as f32,
                0.0,
                -1.0,
                1.0,
            ),

            canvas: ActiveCanvas::Backbuffer,
            backbuffer,

            vertex_data: Vec::with_capacity(MAX_VERTICES * VERTEX_STRIDE),
            element_capacity: MAX_INDICES as i32,
            element_count: 0,

            internal_width,
            internal_height,
            scaling,
            screen_rect,
            letterbox_color: Color::BLACK,

            font_cache,
        })
    }
}

/// A rectangle of `f32`s.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Rectangle {
    /// The X co-ordinate of the rectangle.
    pub x: f32,

    /// The Y co-ordinate of the rectangle.
    pub y: f32,

    /// The width of the rectangle.
    pub width: f32,

    /// The height of the rectangle.
    pub height: f32,
}

impl Rectangle {
    /// Creates a new `Rectangle`.
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns an infinite iterator of horizontally adjecent rectangles, starting at the specified
    /// point and increasing along the X axis.
    ///
    /// This can be useful when slicing spritesheets.
    ///
    /// # Examples
    /// ```
    /// # use tetra::graphics::Rectangle;
    /// let rects: Vec<Rectangle> = Rectangle::row(0.0, 0.0, 16.0, 16.0).take(3).collect();
    ///
    /// assert_eq!(Rectangle::new(0.0, 0.0, 16.0, 16.0), rects[0]);
    /// assert_eq!(Rectangle::new(16.0, 0.0, 16.0, 16.0), rects[1]);
    /// assert_eq!(Rectangle::new(32.0, 0.0, 16.0, 16.0), rects[2]);
    /// ```
    pub fn row(x: f32, y: f32, width: f32, height: f32) -> impl Iterator<Item = Rectangle> {
        RectangleRow {
            next_rect: Rectangle::new(x, y, width, height),
        }
    }

    /// Returns an infinite iterator of vertically adjecent rectangles, starting at the specified
    /// point and increasing along the Y axis.
    ///
    /// This can be useful when slicing spritesheets.
    ///
    /// # Examples
    /// ```
    /// # use tetra::graphics::Rectangle;
    /// let rects: Vec<Rectangle> = Rectangle::column(0.0, 0.0, 16.0, 16.0).take(3).collect();
    ///
    /// assert_eq!(Rectangle::new(0.0, 0.0, 16.0, 16.0), rects[0]);
    /// assert_eq!(Rectangle::new(0.0, 16.0, 16.0, 16.0), rects[1]);
    /// assert_eq!(Rectangle::new(0.0, 32.0, 16.0, 16.0), rects[2]);
    /// ```
    pub fn column(x: f32, y: f32, width: f32, height: f32) -> impl Iterator<Item = Rectangle> {
        RectangleColumn {
            next_rect: Rectangle::new(x, y, width, height),
        }
    }
}

#[derive(Debug, Clone)]
struct RectangleRow {
    next_rect: Rectangle,
}

impl Iterator for RectangleRow {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        let current_rect = self.next_rect;
        self.next_rect.x += self.next_rect.width;
        Some(current_rect)
    }
}

#[derive(Debug, Clone)]
struct RectangleColumn {
    next_rect: Rectangle,
}

impl Iterator for RectangleColumn {
    type Item = Rectangle;

    fn next(&mut self) -> Option<Rectangle> {
        let current_rect = self.next_rect;
        self.next_rect.y += self.next_rect.height;
        Some(current_rect)
    }
}

/// Struct representing the parameters that can be used when drawing.
///
/// You can either use this as a builder by calling `DrawParams::new` and then chaining methods, or
/// construct it manually - whichever you find more pleasant to write.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawParams {
    /// The position that the graphic should be drawn at. Defaults to [0.0, 0.0].
    pub position: Vec2,

    /// The scale that the graphic should be drawn at. Defaults to [1.0, 1.0].
    ///
    /// This can be set to a negative value to flip the graphic around the origin.
    pub scale: Vec2,

    /// The origin of the graphic. Defaults to [0.0, 0.0] (the top left).
    ///
    /// Positioning and scaling will be calculated relative to this point.
    pub origin: Vec2,

    /// The rotation of the graphic, in radians. Defaults to 0.0.
    pub rotation: f32,

    /// A color to multiply the graphic by. Defaults to white.
    pub color: Color,

    /// A sub-region of the graphic to draw. Defaults to `None`, which means the the full graphic will be drawn.
    ///
    /// This is useful if you're using spritesheets (which you should be!).
    pub clip: Option<Rectangle>,
}

impl DrawParams {
    /// Creates a new set of `DrawParams`.
    pub fn new() -> DrawParams {
        DrawParams::default()
    }

    /// Sets the position that the graphic should be drawn at.
    pub fn position(mut self, position: Vec2) -> DrawParams {
        self.position = position;
        self
    }

    /// Sets the scale that the graphic should be drawn at.
    pub fn scale(mut self, scale: Vec2) -> DrawParams {
        self.scale = scale;
        self
    }

    /// Sets the origin of the graphic.
    pub fn origin(mut self, origin: Vec2) -> DrawParams {
        self.origin = origin;
        self
    }

    /// Sets the rotation of the graphic, in radians.
    pub fn rotation(mut self, rotation: f32) -> DrawParams {
        self.rotation = rotation;
        self
    }

    /// Sets the color to multiply the graphic by.
    pub fn color(mut self, color: Color) -> DrawParams {
        self.color = color;
        self
    }

    /// Sets the region of the graphic to draw.
    pub fn clip(mut self, clip: Rectangle) -> DrawParams {
        self.clip = Some(clip);
        self
    }
}

impl Default for DrawParams {
    fn default() -> DrawParams {
        DrawParams {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            color: Color::WHITE,
            clip: None,
        }
    }
}

impl From<Vec2> for DrawParams {
    fn from(position: Vec2) -> DrawParams {
        DrawParams {
            position,
            ..DrawParams::default()
        }
    }
}

/// Represents the different filtering algorithms that can be used when scaling an image.
///
/// Tetra currently defaults to using `Nearest` for all newly created textures.
#[derive(Debug, Clone, Copy)]
pub enum FilterMode {
    /// Nearest-neighbor interpolation. This preserves hard edges and details, but may look pixelated.
    ///
    /// If you're using pixel art, this is probably the scaling mode you should use.
    Nearest,

    /// Linear interpolation. This smooths images when scaling them up or down.
    Linear,
}

/// Represents a type that can be drawn.
///
/// [`graphics::draw`](fn.draw.html) can be used to draw without importing this trait, which is sometimes
/// more convienent.
pub trait Drawable {
    /// Draws `self` to the screen (or a canvas, if one is enabled), using the specified parameters.
    ///
    /// Any type that implements `Into<DrawParams>` can be passed into this method. For example, since the majority
    /// of the time, you only care about changing the position, a `Vec2` can be passed to set the position and leave
    /// everything else as the defaults.
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>;
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
    let wrapped_texture = Some(texture);

    if wrapped_texture != ctx.graphics.texture {
        flush(ctx);
        ctx.graphics.texture = wrapped_texture;
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
    set_canvas_ex(ctx, ActiveCanvas::Backbuffer);
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
            ActiveCanvas::Backbuffer => {
                ctx.gl.bind_canvas(Some(&ctx.graphics.backbuffer));
                ctx.gl.front_face(FrontFace::Clockwise);
                ctx.gl.viewport(
                    0,
                    0,
                    ctx.graphics.backbuffer.width(),
                    ctx.graphics.backbuffer.height(),
                );
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
            None => return,
            Some(ActiveTexture::Backbuffer) => &ctx.graphics.backbuffer.texture,
            Some(ActiveTexture::FontCache) => &ctx.graphics.font_cache_texture,
            Some(ActiveTexture::User(t)) => &t,
        };

        let shader = match &ctx.graphics.shader {
            ActiveShader::Default => &ctx.graphics.default_shader,
            ActiveShader::User(s) => &s,
        };

        let projection = match &ctx.graphics.canvas {
            ActiveCanvas::Window => &ctx.graphics.window_projection,
            ActiveCanvas::Backbuffer => &ctx.graphics.backbuffer.projection,
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
    set_canvas_ex(ctx, ActiveCanvas::Window);
    set_shader_ex(ctx, ActiveShader::Default);
    set_texture_ex(ctx, ActiveTexture::Backbuffer);

    clear(ctx, ctx.graphics.letterbox_color);

    let screen_rect = ctx.graphics.screen_rect;

    push_quad(
        ctx,
        screen_rect.x,
        screen_rect.y,
        screen_rect.x + screen_rect.width,
        screen_rect.y + screen_rect.height,
        0.0,
        0.0,
        1.0,
        1.0,
        &DrawParams::new(),
    );

    flush(ctx);

    platform::swap_buffers(ctx);

    set_canvas_ex(ctx, ActiveCanvas::Backbuffer);
}

/// Gets the internal width of the screen.
pub fn get_internal_width(ctx: &Context) -> i32 {
    ctx.graphics.backbuffer.width()
}

/// Sets the internal width of the screen.
///
/// If the scaling mode is set to Resize, this will not take effect until the scaling
/// mode is changed.
pub fn set_internal_width(ctx: &mut Context, width: i32) {
    set_internal_size(ctx, width, ctx.graphics.internal_height);
}

/// Gets the internal height of the screen.
pub fn get_internal_height(ctx: &Context) -> i32 {
    ctx.graphics.backbuffer.height()
}

/// Sets the internal height of the screen.
///
/// If the scaling mode is set to Resize, this will not take effect until the scaling
/// mode is changed.
pub fn set_internal_height(ctx: &mut Context, height: i32) {
    set_internal_size(ctx, ctx.graphics.internal_width, height);
}

/// Gets the internal size of the screen.
pub fn get_internal_size(ctx: &Context) -> (i32, i32) {
    (
        ctx.graphics.backbuffer.width(),
        ctx.graphics.backbuffer.height(),
    )
}

/// Sets the internal size of the screen.
///
/// If the scaling mode is set to Resize, this will not take effect until the scaling
/// mode is changed.
pub fn set_internal_size(ctx: &mut Context, width: i32, height: i32) {
    ctx.graphics.internal_width = width;
    ctx.graphics.internal_height = height;

    if let ScreenScaling::Resize = ctx.graphics.scaling {

    } else {
        set_backbuffer_size(ctx, width, height);
        update_screen_rect(
            ctx,
            ctx.graphics.internal_width,
            ctx.graphics.internal_height,
            window::get_width(ctx),
            window::get_height(ctx),
        );
    }
}

pub(crate) fn get_screen_rect(ctx: &Context) -> Rectangle {
    ctx.graphics.screen_rect
}

/// Gets the current scaling mode.
pub fn get_scaling(ctx: &mut Context) -> ScreenScaling {
    ctx.graphics.scaling
}

/// Sets the current scaling mode.
pub fn set_scaling(ctx: &mut Context, scaling: ScreenScaling) {
    ctx.graphics.scaling = scaling;

    if let ScreenScaling::Resize = ctx.graphics.scaling {
        set_backbuffer_size(ctx, window::get_width(ctx), window::get_height(ctx));
    } else {
        set_backbuffer_size(
            ctx,
            ctx.graphics.internal_width,
            ctx.graphics.internal_height,
        );
    }

    update_screen_rect(
        ctx,
        ctx.graphics.internal_width,
        ctx.graphics.internal_height,
        window::get_width(ctx),
        window::get_height(ctx),
    );
}

/// Sets the color of the letterbox bars that are displayed when scaling the screen.
///
/// For information on which scaling modes can cause letterboxing, see the docs for
/// [`ScreenScaling`](./scaling/enum.ScreenScaling.html).
pub fn set_letterbox_color(ctx: &mut Context, color: Color) {
    ctx.graphics.letterbox_color = color;
}

/// Returns the filter mode that will be used by newly created textures and canvases.
pub fn get_default_filter_mode(ctx: &Context) -> FilterMode {
    ctx.gl.get_default_filter_mode()
}

/// Sets the filter mode that will be used by newly created textures and canvases.
pub fn set_default_filter_mode(ctx: &mut Context, filter_mode: FilterMode) {
    ctx.gl.set_default_filter_mode(filter_mode);
}

pub(crate) fn set_window_projection(ctx: &mut Context, width: i32, height: i32) {
    ctx.graphics.window_projection = glm::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

    if let ScreenScaling::Resize = get_scaling(ctx) {
        set_backbuffer_size(ctx, width, height);
    }

    update_screen_rect(
        ctx,
        ctx.graphics.internal_width,
        ctx.graphics.internal_height,
        width,
        height,
    );
}

pub(crate) fn set_backbuffer_size(ctx: &mut Context, width: i32, height: i32) {
    if ctx.graphics.backbuffer.width() != width || ctx.graphics.backbuffer.height() != height {
        ctx.graphics.backbuffer = Canvas::new(ctx, width, height);

        if let ActiveCanvas::Backbuffer = ctx.graphics.canvas {
            ctx.gl.viewport(0, 0, width, height);
        }
    }
}

pub(crate) fn update_screen_rect(
    ctx: &mut Context,
    internal_width: i32,
    internal_height: i32,
    window_width: i32,
    window_height: i32,
) {
    ctx.graphics.screen_rect = ctx.graphics.scaling.get_screen_rect(
        internal_width,
        internal_height,
        window_width,
        window_height,
    );
}
