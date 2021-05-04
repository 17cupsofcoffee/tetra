use std::rc::Rc;

use crate::error::Result;
use crate::graphics::{DrawParams, FilterMode, Texture};
use crate::platform::{GraphicsDevice, RawFramebuffer, RawRenderbuffer};
use crate::Context;

use super::ImageData;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct CanvasSettings {
    pub samples: u8,
    pub enable_stencil_buffer: bool,
}

impl Default for CanvasSettings {
    fn default() -> Self {
        Self {
            samples: 0,
            enable_stencil_buffer: false,
        }
    }
}

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
/// Creating a `Canvas` is a relatively expensive operation. If you can, store them in your
/// [`State`](crate::State) struct rather than recreating them each frame.
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
    pub(crate) framebuffer: Rc<RawFramebuffer>,
    pub(crate) texture: Texture,
    pub(crate) stencil_buffer: Option<Texture>,
    pub(crate) multisample: Option<Rc<RawRenderbuffer>>,
}

impl Canvas {
    /// Creates a new canvas.
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the underlying
    /// graphics API encounters an error.
    pub fn new(
        ctx: &mut Context,
        width: i32,
        height: i32,
        settings: CanvasSettings,
    ) -> Result<Canvas> {
        Canvas::with_device(
            &mut ctx.device,
            width,
            height,
            ctx.graphics.default_filter_mode,
            settings,
        )
    }

    pub(crate) fn with_device(
        device: &mut GraphicsDevice,
        width: i32,
        height: i32,
        filter_mode: FilterMode,
        settings: CanvasSettings,
    ) -> Result<Canvas> {
        let texture = device.new_texture(width, height, filter_mode)?;

        let framebuffer = device.new_framebuffer()?;
        device.attach_texture_to_framebuffer(&framebuffer, &texture, true, true);

        let stencil_buffer = if settings.enable_stencil_buffer {
            let stencil_buffer = device.new_stencil_buffer(width, height, filter_mode)?;
            device.attach_stencil_buffer_to_framebuffer(&framebuffer, &stencil_buffer, true, true);
            Some(Texture::from_raw(stencil_buffer, filter_mode))
        } else {
            None
        };

        let multisample = if settings.samples > 0 {
            let multisample = device.new_renderbuffer(width, height, settings.samples)?;
            device.attach_renderbuffer_to_framebuffer(&framebuffer, &multisample, true, true);

            Some(Rc::new(multisample))
        } else {
            None
        };

        Ok(Canvas {
            framebuffer: Rc::new(framebuffer),
            texture: Texture::from_raw(texture, filter_mode),
            stencil_buffer,
            multisample,
        })
    }

    /// Draws the canvas to the screen (or to another canvas, if one is enabled).
    pub fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        self.texture.draw(ctx, params)
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

    /// Gets the canvas' data from the GPU.
    ///
    /// This can be useful if you need to do some image processing on the CPU,
    /// or if you want to output the image data somewhere. This is a fairly
    /// slow operation, so avoid doing it too often!
    ///
    /// If this is the currently active canvas, you should unbind it or call
    /// [`graphics::flush`](super::flush) before calling this method, to ensure all
    /// pending draw calls are reflected in the output. Similarly, if the canvas is
    /// multisampled, it must be [resolved](#resolving) before
    /// changes will be reflected in this method's output.
    pub fn get_data(&self, ctx: &mut Context) -> ImageData {
        self.texture.get_data(ctx)
    }

    /// Writes RGBA pixel data to a specified region of the canvas.
    ///
    /// This method requires you to provide enough data to fill the target rectangle.
    /// If you provide too little data, an error will be returned.
    /// If you provide too much data, it will be truncated.
    ///
    /// If you want to overwrite the entire canvas, the `replace_data` method offers a
    /// more concise way of doing this.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NotEnoughData`](crate::TetraError::NotEnoughData) will be returned
    /// if not enough data is provided to fill the target rectangle. This is to prevent
    /// the graphics API from trying to read uninitialized memory.
    ///
    /// # Panics
    ///
    /// Panics if any part of the target rectangle is outside the bounds of the canvas.
    pub fn set_data(
        &self,
        ctx: &mut Context,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        data: &[u8],
    ) -> Result {
        self.texture.set_data(ctx, x, y, width, height, data)
    }

    /// Overwrites the entire canvas with new RGBA pixel data.
    ///
    /// This method requires you to provide enough data to fill the canvas.
    /// If you provide too little data, an error will be returned.
    /// If you provide too much data, it will be truncated.
    ///
    /// If you only want to write to a subsection of the canvas, use the `set_data`
    /// method instead.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NotEnoughData`](crate::TetraError::NotEnoughData) will be returned
    /// if not enough data is provided to fill the target rectangle. This is to prevent
    /// the graphics API from trying to read uninitialized memory.
    pub fn replace_data(&self, ctx: &mut Context, data: &[u8]) -> Result {
        self.texture.replace_data(ctx, data)
    }

    /// Returns a reference to the canvas' underlying texture.
    ///
    /// If this is the currently active canvas, you may want to unbind it or call
    /// [`graphics::flush`](super::flush) before trying to access the underlying
    /// texture data, to ensure all pending draw calls are completed. Similarly,
    /// if the canvas is multisampled, it must be [resolved](#resolving)
    /// before changes will be reflected in the texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}
