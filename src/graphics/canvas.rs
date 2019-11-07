use std::rc::Rc;

use crate::error::Result;
use crate::graphics::opengl::{GLDevice, GLFramebuffer};
use crate::graphics::{DrawParams, Drawable, FilterMode, Texture};
use crate::math::{FrustumPlanes, Mat4};
use crate::Context;

/// A 2D texture that can be used for off-screen rendering.
///
/// This is sometimes referred to as a 'render texture' or 'render target' in other
/// frameworks.
///
/// Canvases can be useful if you want to do some rendering upfront and then cache the result
/// (e.g. a static background), or if you want to apply transformations/shaders to multiple
/// things simultaneously.
///
/// Note that creating a canvas is a relatively expensive operation! You rarely (if ever) should
/// create them in your `draw` or `update` methods. Instead, add it as a member of your `State`
/// struct.
///
/// # Examples
///
/// ```rust
/// # use tetra::{Context, State};
/// # use tetra::graphics::{self, Color, Canvas};
/// # use tetra::math::Vec2;
/// #
/// struct GameState {
///     canvas: Canvas,
/// }
///
/// impl GameState {
///     fn new(ctx: &mut Context) -> tetra::Result<GameState> {
///         Ok(GameState {
///             canvas: Canvas::new(ctx, 640, 480)?
///         })
///     }
/// }
///
/// impl State for GameState {
///     fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
///         graphics::set_canvas(ctx, &self.canvas);
///
///         // Draw some stuff to the canvas here, using the normal graphics API.
///
///         // When you're done, reset the canvas:
///         graphics::reset_canvas(ctx);
///
///         // Now you can draw the canvas to the screen:
///         graphics::clear(ctx, Color::BLACK);
///         graphics::draw(ctx, &self.canvas, Vec2::new(0.0, 0.0));
///
///         Ok(())
///     }
/// }
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub(crate) texture: Texture,
    pub(crate) framebuffer: Rc<GLFramebuffer>,
    pub(crate) projection: Mat4<f32>,
}

impl Canvas {
    /// Creates a new canvas.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    pub fn new(ctx: &mut Context, width: i32, height: i32) -> Result<Canvas> {
        Canvas::with_device(&mut ctx.gl, width, height)
    }

    pub(crate) fn with_device(gl: &mut GLDevice, width: i32, height: i32) -> Result<Canvas> {
        let texture = Texture::with_device_empty(gl, width, height)?;
        let framebuffer = gl.new_framebuffer(&*texture.handle.borrow(), true)?;

        Ok(Canvas {
            texture,
            framebuffer: Rc::new(framebuffer),
            projection: Mat4::orthographic_rh_no(FrustumPlanes {
                left: 0.0,
                right: width as f32,
                bottom: 0.0,
                top: height as f32,
                near: -1.0,
                far: 1.0,
            }),
        })
    }

    /// Returns the width of the canvas.
    pub fn width(&self) -> i32 {
        self.texture.width()
    }

    /// Returns the height of the canvas.
    pub fn height(&self) -> i32 {
        self.texture.height()
    }

    /// Returns the size of the canvas.
    pub fn size(&self) -> (i32, i32) {
        self.texture.size()
    }

    /// Returns the filter mode being used by the canvas.
    pub fn filter_mode(&self) -> FilterMode {
        self.texture.filter_mode()
    }

    /// Sets the filter mode that should be used by the canvas.
    pub fn set_filter_mode(&mut self, ctx: &mut Context, filter_mode: FilterMode) {
        self.texture.set_filter_mode(ctx, filter_mode);
    }

    /// Returns the canvas' underlying texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

impl Drawable for Canvas {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        self.texture.draw(ctx, params)
    }
}
