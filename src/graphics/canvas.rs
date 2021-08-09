use std::rc::Rc;

use crate::error::Result;
use crate::graphics::{DrawParams, FilterMode, Texture};
use crate::platform::{RawCanvas, RawRenderbuffer};
use crate::Context;

use super::ImageData;

/// A builder for creating advanced canvas configurations.
///
/// By default, Tetra's canvases are fairly simple - they just provide a [`Texture`] that you
/// can render things to. However, they can also be configured with extra features via this
/// builder, such as multisampling and additional buffers.
#[derive(Debug, Clone)]
pub struct CanvasBuilder {
    width: i32,
    height: i32,
    samples: u8,
    stencil_buffer: bool,
    hdr: bool,
}

impl CanvasBuilder {
    /// Creates a new canvas builder, which can be used to create a canvas with multisampling and/or
    /// additional buffers.
    ///
    /// You can also use [`Canvas::builder`] as a shortcut for this, if you want
    /// to avoid the extra import.
    pub fn new(width: i32, height: i32) -> CanvasBuilder {
        CanvasBuilder {
            width,
            height,
            samples: 0,
            stencil_buffer: false,
            hdr: false,
        }
    }

    /// Sets the level of multisample anti-aliasing to use.
    ///
    /// The number of samples that can be used varies between graphics cards - `2`, `4` and `8` are reasonably
    /// well supported. When set to `0` (the default), no multisampling will be used.
    ///
    /// # Resolving
    ///
    /// In order to actually display a multisampled canvas, it first has to be downsampled (or 'resolved'). This is
    /// done automatically once you switch to a different canvas/the backbuffer. Until this step takes place,
    /// your rendering will *not* be reflected in the canvas' underlying [`texture`](Canvas::texture) (and by
    /// extension, in the output of [`draw`](Canvas::draw) and [`get_data`](Canvas::get_data)).
    pub fn samples(&mut self, samples: u8) -> &mut CanvasBuilder {
        self.samples = samples;
        self
    }

    /// Sets whether the canvas should have a stencil buffer.
    ///
    /// Setting this to `true` allows you to use stencils while rendering to the canvas, at the cost
    /// of some extra video RAM usage.
    pub fn stencil_buffer(&mut self, enabled: bool) -> &mut CanvasBuilder {
        self.stencil_buffer = enabled;
        self
    }

    /// Sets whether the canvas should support HDR.
    ///
    /// Setting this to `true` allows you to store color values greater than 1.0, at the cost
    /// of some extra video RAM usage.
    pub fn hdr(&mut self, enabled: bool) -> &mut CanvasBuilder {
        self.hdr = enabled;
        self
    }

    /// Builds the canvas.
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the underlying
    /// graphics API encounters an error.
    pub fn build(&self, ctx: &mut Context) -> Result<Canvas> {
        let attachments = ctx.device.new_canvas(
            self.width,
            self.height,
            ctx.graphics.default_filter_mode,
            self.samples,
            self.stencil_buffer,
            self.hdr,
        )?;

        Ok(Canvas {
            handle: Rc::new(attachments.canvas),
            texture: Texture::from_raw(attachments.color, ctx.graphics.default_filter_mode),
            stencil_buffer: attachments.depth_stencil.map(Rc::new),
            multisample: attachments.multisample_color.map(Rc::new),
        })
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
/// Creating a canvas is quite an expensive operation. Try to reuse them, rather
/// than recreating them every frame.
///
/// Switching which canvas you are rendering to can be also be slow, as it requires flushing
/// any pending draw calls to the GPU. It's usually a good idea to do your rendering
/// to a canvas all in one go, if you can.
///
/// You can clone a canvas cheaply, as it is a [reference-counted](https://doc.rust-lang.org/std/rc/struct.Rc.html)
/// handle to a GPU resource. However, this does mean that modifying a canvas (e.g.
/// drawing to it) will also affect any clones that exist of it.
///
/// # Examples
///
/// The [`canvas`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/canvas.rs)
/// example demonstrates how to draw to a canvas, and then draw that canvas to
/// the screen.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub(crate) handle: Rc<RawCanvas>,
    pub(crate) texture: Texture,
    pub(crate) stencil_buffer: Option<Rc<RawRenderbuffer>>,
    pub(crate) multisample: Option<Rc<RawRenderbuffer>>,
}

impl Canvas {
    /// Creates a new canvas, with the default settings (no multisampling, no additional buffers).
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the underlying
    /// graphics API encounters an error.
    pub fn new(ctx: &mut Context, width: i32, height: i32) -> Result<Canvas> {
        CanvasBuilder::new(width, height).build(ctx)
    }

    /// Creates a new canvas builder, which can be used to create a canvas with multisampling and/or
    /// additional buffers.
    pub fn builder(width: i32, height: i32) -> CanvasBuilder {
        CanvasBuilder::new(width, height)
    }

    /// Creates a new canvas, with the specified level of multisample anti-aliasing.
    ///
    /// The number of samples that can be used varies between graphics cards - `2`, `4` and `8` are reasonably
    /// well supported. When set to `0` (the default), no multisampling will be used.
    ///
    /// # Resolving
    ///
    /// In order to actually display a multisampled canvas, it first has to be downsampled (or 'resolved'). This is
    /// done automatically once you switch to a different canvas/the backbuffer. Until this step takes place,
    /// your rendering will *not* be reflected in the canvas' underlying [`texture`](Self::texture) (and by
    /// extension, in the output of [`draw`](Self::draw) and [`get_data`](Self::get_data)).
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the underlying
    /// graphics API encounters an error.
    #[deprecated(since = "0.6.4", note = "use Canvas::builder instead")]
    pub fn multisampled(ctx: &mut Context, width: i32, height: i32, samples: u8) -> Result<Canvas> {
        CanvasBuilder::new(width, height)
            .samples(samples)
            .build(ctx)
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
