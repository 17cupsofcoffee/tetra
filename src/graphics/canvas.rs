use std::rc::Rc;

use crate::error::Result;
use crate::graphics::{DrawParams, Drawable, FilterMode, Texture};
use crate::platform::{GraphicsDevice, RawFramebuffer};
use crate::Context;

/// A texture that can be used for off-screen rendering.
///
/// This is sometimes referred to as a 'render texture' or 'render target' in other
/// frameworks.
///
/// Canvases can be useful if you want to do some rendering upfront and then cache the result
/// (e.g. a static background), or if you want to apply transformations/shaders to multiple
/// things simultaneously.
///
/// # Performance
///
/// Creating a `Canvas` is a relatively expensive operation. If you can, store them in your `State`
/// struct rather than recreating them each frame.
///
/// Cloning a `Canvas` is a very cheap operation, as the underlying data is shared between the
/// original instance and the clone via [reference-counting](https://doc.rust-lang.org/std/rc/struct.Rc.html).
/// This does mean, however, that updating a `Canvas` (for example, changing its filter mode) will also
/// update any other clones of that `Canvas`.
///
/// # Examples
///
/// The [`canvas`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/canvas.rs)
/// example demonstrates how to draw to a canvas, and then draw that canvas to
/// the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub(crate) texture: Texture,
    pub(crate) framebuffer: Rc<RawFramebuffer>,
}

impl Canvas {
    /// Creates a new canvas.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    pub fn new(ctx: &mut Context, width: i32, height: i32) -> Result<Canvas> {
        Canvas::with_device(
            &mut ctx.device,
            width,
            height,
            ctx.graphics.default_filter_mode,
        )
    }

    pub(crate) fn with_device(
        device: &mut GraphicsDevice,
        width: i32,
        height: i32,
        filter_mode: FilterMode,
    ) -> Result<Canvas> {
        let texture = Texture::with_device_empty(device, width, height, filter_mode)?;
        let framebuffer = device.new_framebuffer(&texture.data.handle, true)?;

        Ok(Canvas {
            texture,
            framebuffer: Rc::new(framebuffer),
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

    /// Returns a reference to the canvas' underlying texture.
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
