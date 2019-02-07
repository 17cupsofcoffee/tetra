//! Functions and types used for rendering to the screen.
//!
//! This module implements a (hopefully!) efficent quad renderer, which will queue up
//! drawing operations until it is absolutely necessary to send them to the graphics
//! hardware. This allows us to minimize the number of draw calls made, speeding up
//! rendering.

pub mod animation;
pub mod color;
pub(crate) mod opengl;
pub mod scaling;
pub mod shader;
pub mod text;
pub mod texture;
pub mod ui;

pub use self::animation::Animation;
pub use self::color::Color;
pub use self::scaling::ScreenScaling;
pub use self::shader::Shader;
pub use self::text::{Font, Text};
pub use self::texture::Texture;
pub use self::ui::NineSlice;
pub use glm::Vec2;

use glm::{self, Mat3, Mat4};
use glyph_brush::{GlyphBrush, GlyphBrushBuilder};

use crate::error::Result;
use crate::graphics::opengl::{
    BufferUsage, GLDevice, GLFramebuffer, GLIndexBuffer, GLVertexBuffer, TextureFormat,
};
use crate::window;
use crate::Context;

const MAX_SPRITES: usize = 2048;
const MAX_VERTICES: usize = MAX_SPRITES * 4;
const MAX_INDICES: usize = MAX_SPRITES * 6;
const VERTEX_STRIDE: usize = 8;
const INDEX_ARRAY: [u32; 6] = [0, 1, 2, 2, 3, 0];
const DEFAULT_FONT: &[u8] = include_bytes!("./resources/DejaVuSansMono.ttf");

#[derive(PartialEq)]
pub(crate) enum ActiveTexture {
    Framebuffer,
    FontCache,
    User(Texture),
}

#[derive(PartialEq)]
pub(crate) enum ActiveShader {
    Default,
    User(Shader),
}

#[derive(PartialEq)]
pub(crate) enum ActiveProjection {
    Internal,
    Window,
}

#[derive(PartialEq)]
pub(crate) enum ActiveFramebuffer {
    Backbuffer,
    Window,
}

pub(crate) struct GraphicsContext {
    vertex_buffer: GLVertexBuffer,
    index_buffer: GLIndexBuffer,

    texture: Option<ActiveTexture>,
    backbuffer_texture: Texture,
    font_cache_texture: Texture,

    shader: ActiveShader,
    default_shader: Shader,

    projection: ActiveProjection,
    internal_projection: Mat4,
    window_projection: Mat4,

    framebuffer: ActiveFramebuffer,
    backbuffer: GLFramebuffer,

    vertex_data: Vec<f32>,
    element_capacity: usize,
    element_count: usize,

    internal_width: i32,
    internal_height: i32,
    scaling: ScreenScaling,
    screen_rect: Rectangle,

    font_cache: GlyphBrush<'static>,
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
        assert!(
            MAX_VERTICES <= 32767,
            "Can't have more than 32767 vertices to a single buffer"
        );

        let (backbuffer_width, backbuffer_height) = match scaling {
            ScreenScaling::Resize => (window_width, window_height),
            _ => (internal_width, internal_height),
        };

        let screen_rect =
            scaling.get_screen_rect(internal_width, internal_height, window_width, window_height);

        let backbuffer = device.new_framebuffer();
        let backbuffer_texture = Texture::from_handle(device.new_texture(
            backbuffer_width,
            backbuffer_height,
            TextureFormat::Rgb,
        ));

        device.attach_texture_to_framebuffer(&backbuffer, &backbuffer_texture.handle, false);
        device.set_viewport(0, 0, backbuffer_width, backbuffer_height);

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
        );

        device.set_vertex_buffer_attribute(&vertex_buffer, 0, 2, 0);
        device.set_vertex_buffer_attribute(&vertex_buffer, 1, 2, 2);
        device.set_vertex_buffer_attribute(&vertex_buffer, 2, 4, 4);

        let index_buffer = device.new_index_buffer(MAX_INDICES, BufferUsage::StaticDraw);

        device.set_index_buffer_data(&index_buffer, &indices, 0);

        let default_shader = Shader::from_handle(device.compile_program(
            shader::DEFAULT_VERTEX_SHADER,
            shader::DEFAULT_FRAGMENT_SHADER,
        )?);

        let font_cache = GlyphBrushBuilder::using_font_bytes(DEFAULT_FONT).build();
        let (width, height) = font_cache.texture_dimensions();

        let font_cache_texture = Texture::from_handle(device.new_texture(
            width as i32,
            height as i32,
            TextureFormat::Rgba,
        ));

        Ok(GraphicsContext {
            vertex_buffer,
            index_buffer,

            texture: None,
            backbuffer_texture,
            font_cache_texture,

            shader: ActiveShader::Default,
            default_shader,

            projection: ActiveProjection::Internal,
            internal_projection: ortho(
                0.0,
                backbuffer_width as f32,
                backbuffer_height as f32,
                0.0,
                -1.0,
                1.0,
            ),
            window_projection: ortho(
                0.0,
                window_width as f32,
                window_height as f32,
                0.0,
                -1.0,
                1.0,
            ),

            framebuffer: ActiveFramebuffer::Backbuffer,
            backbuffer,

            vertex_data: Vec::with_capacity(MAX_VERTICES * VERTEX_STRIDE),
            element_capacity: MAX_INDICES,
            element_count: 0,

            internal_width,
            internal_height,
            scaling,
            screen_rect,

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
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Rectangle {
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

    #[deprecated(
        since = "0.2.6",
        note = "This was only intended for internal use, but was made public by mistake."
    )]
    #[doc(hidden)]
    pub fn build_matrix(&self) -> Mat3 {
        glm::translation2d(&self.position)
            * glm::rotation2d(self.rotation)
            * glm::scaling2d(&self.scale)
            * glm::translation2d(&-self.origin)
    }
}

impl Default for DrawParams {
    fn default() -> DrawParams {
        DrawParams {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            color: color::WHITE,
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

/// Represents a type that can be drawn to the screen/render target.
///
/// [graphics::draw](fn.draw.html) can be used to draw without importing this trait, which is sometimes
/// more convienent.
pub trait Drawable {
    /// Draws `self` to the currently enabled render target, using the specified parameters.
    ///
    /// Any type that implements `Into<DrawParams>` can be passed into this method. For example, since the majority
    /// of the time, you only care about changing the position, a `Vec2` can be passed to set the position and leave
    /// everything else as the defaults.
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>;
}

/// Clears the currently enabled render target to the specified color.
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
    let (ox1, oy1, ox2, oy2, ox3, oy3, ox4, oy4) = if params.rotation != 0.0 {
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
    } else {
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

/// Draws an object to the currently enabled render target.
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

pub(crate) fn set_shader_ex(ctx: &mut Context, shader: ActiveShader) -> Option<Shader> {
    if shader != ctx.graphics.shader {
        flush(ctx);
        let old_shader = std::mem::replace(&mut ctx.graphics.shader, shader);

        if let ActiveShader::User(s) = old_shader {
            return Some(s);
        }
    }

    None
}

pub(crate) fn set_projection_ex(ctx: &mut Context, projection: ActiveProjection) {
    if projection != ctx.graphics.projection {
        flush(ctx);
        ctx.graphics.projection = projection;
    }
}

pub(crate) fn set_framebuffer_ex(ctx: &mut Context, framebuffer: ActiveFramebuffer) {
    if framebuffer != ctx.graphics.framebuffer {
        flush(ctx);
        ctx.graphics.framebuffer = framebuffer;

        match ctx.graphics.framebuffer {
            ActiveFramebuffer::Backbuffer => {
                ctx.gl.bind_framebuffer(&ctx.graphics.backbuffer);
                ctx.gl
                    .set_viewport(0, 0, get_internal_width(ctx), get_internal_height(ctx));
            }
            ActiveFramebuffer::Window => {
                ctx.gl.bind_default_framebuffer();
                ctx.gl
                    .set_viewport(0, 0, window::get_width(ctx), window::get_height(ctx));
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
            Some(ActiveTexture::Framebuffer) => &ctx.graphics.backbuffer_texture,
            Some(ActiveTexture::FontCache) => &ctx.graphics.font_cache_texture,
            Some(ActiveTexture::User(t)) => &t,
        };

        let shader = match &ctx.graphics.shader {
            ActiveShader::Default => &ctx.graphics.default_shader,
            ActiveShader::User(s) => &s,
        };

        let projection = match &ctx.graphics.projection {
            ActiveProjection::Internal => &ctx.graphics.internal_projection,
            ActiveProjection::Window => &ctx.graphics.window_projection,
        };

        ctx.gl.bind_texture(&texture.handle);

        ctx.gl.bind_program(&shader.handle);
        ctx.gl
            .set_uniform(&shader.handle, "u_projection", &projection);

        ctx.gl.bind_vertex_buffer(&ctx.graphics.vertex_buffer);
        ctx.gl
            .set_vertex_buffer_data(&ctx.graphics.vertex_buffer, &ctx.graphics.vertex_data, 0);

        ctx.gl
            .draw_elements(&ctx.graphics.index_buffer, ctx.graphics.element_count);

        ctx.graphics.vertex_data.clear();
        ctx.graphics.element_count = 0;
    }
}

/// Draws the currently enabled render target to the screen, scaling/letterboxing it if necessary.
///
/// You usually will not have to call this manually, as it is called for you at the end of every
/// frame. Note that calling it will trigger a [`flush`](fn.flush.html) to the graphics hardware.
pub fn present(ctx: &mut Context) {
    set_framebuffer_ex(ctx, ActiveFramebuffer::Window);
    set_projection_ex(ctx, ActiveProjection::Window);
    set_texture_ex(ctx, ActiveTexture::Framebuffer);
    let user_shader = set_shader_ex(ctx, ActiveShader::Default);

    clear(ctx, color::BLACK);

    let screen_rect = ctx.graphics.screen_rect;

    push_quad(
        ctx,
        screen_rect.x,
        screen_rect.y,
        screen_rect.x + screen_rect.width,
        screen_rect.y + screen_rect.height,
        0.0,
        1.0,
        1.0,
        0.0,
        &DrawParams::new(),
    );

    flush(ctx);
    ctx.window.gl_swap_window();

    set_framebuffer_ex(ctx, ActiveFramebuffer::Backbuffer);
    set_projection_ex(ctx, ActiveProjection::Internal);

    if let Some(s) = user_shader {
        set_shader_ex(ctx, ActiveShader::User(s));
    }
}

/// Gets the internal width of the screen.
pub fn get_internal_width(ctx: &Context) -> i32 {
    ctx.graphics.backbuffer_texture.width()
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
    ctx.graphics.backbuffer_texture.height()
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
        ctx.graphics.backbuffer_texture.width(),
        ctx.graphics.backbuffer_texture.height(),
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
        update_screen_rect(ctx);
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

    update_screen_rect(ctx);
}

pub(crate) fn set_backbuffer_size(ctx: &mut Context, width: i32, height: i32) {
    ctx.graphics.internal_projection = ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);

    ctx.graphics.backbuffer_texture =
        Texture::from_handle(ctx.gl.new_texture(width, height, TextureFormat::Rgb));

    ctx.gl.attach_texture_to_framebuffer(
        &ctx.graphics.backbuffer,
        &ctx.graphics.backbuffer_texture.handle,
        true,
    );

    // TODO: This might conflict with set_framebuffer_ex - bit of a hack
    ctx.gl.set_viewport(0, 0, width, height);
}

pub(crate) fn update_screen_rect(ctx: &mut Context) {
    ctx.graphics.screen_rect = ctx.graphics.scaling.get_screen_rect(
        ctx.graphics.backbuffer_texture.width(),
        ctx.graphics.backbuffer_texture.height(),
        window::get_width(ctx),
        window::get_height(ctx),
    );
}

pub(crate) fn set_window_projection(ctx: &mut Context, width: i32, height: i32) {
    ctx.graphics.window_projection = ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
}

pub(crate) fn ortho(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    // Taken from GGEZ - nalgebra doesn't like upside-down projections
    let c0r0 = 2.0 / (right - left);
    let c0r1 = 0.0;
    let c0r2 = 0.0;
    let c0r3 = 0.0;
    let c1r0 = 0.0;
    let c1r1 = 2.0 / (top - bottom);
    let c1r2 = 0.0;
    let c1r3 = 0.0;
    let c2r0 = 0.0;
    let c2r1 = 0.0;
    let c2r2 = -2.0 / (far - near);
    let c2r3 = 0.0;
    let c3r0 = -(right + left) / (right - left);
    let c3r1 = -(top + bottom) / (top - bottom);
    let c3r2 = -(far + near) / (far - near);
    let c3r3 = 1.0;

    Mat4::from([
        [c0r0, c0r1, c0r2, c0r3],
        [c1r0, c1r1, c1r2, c1r3],
        [c2r0, c2r1, c2r2, c2r3],
        [c3r0, c3r1, c3r2, c3r3],
    ])
}
