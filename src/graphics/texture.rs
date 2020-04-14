//! Functions and types relating to textures.

use std::cell::Cell;
use std::path::Path;
use std::rc::Rc;

use crate::error::{Result, TetraError};
use crate::fs;
use crate::graphics::{self, DrawParams, Drawable, Rectangle};
use crate::platform::{GraphicsDevice, RawTexture};
use crate::Context;

#[derive(Debug)]
pub(crate) struct TextureSharedData {
    pub(crate) handle: RawTexture,
    filter_mode: Cell<FilterMode>,
}

impl PartialEq for TextureSharedData {
    fn eq(&self, other: &TextureSharedData) -> bool {
        // filter_mode should always match what's set on the GPU,
        // so we can ignore it for equality checks.

        self.handle.eq(&other.handle)
    }
}

/// A texture, held in GPU memory.
///
/// The following file formats are supported:
///
/// * PNG
/// * JPEG
/// * GIF
/// * BMP
/// * TIFF
/// * TGA
/// * WEBP
/// * ICO
/// * PNM
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
#[derive(Debug, Clone, PartialEq)]
pub struct Texture {
    pub(crate) data: Rc<TextureSharedData>,
}

impl Texture {
    /// Creates a new texture from the given file.
    ///
    /// The format will be determined based on the file extension.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::FailedToLoadAsset` will be returned if the file could not be loaded.
    /// * `TetraError::InvalidTexture` will be returned if the texture data was invalid.
    pub fn new<P>(ctx: &mut Context, path: P) -> Result<Texture>
    where
        P: AsRef<Path>,
    {
        let image = fs::read_to_image(path)?.to_rgba();
        let (width, height) = image.dimensions();

        Texture::from_rgba(
            ctx,
            width as i32,
            height as i32,
            image.into_raw().as_slice(),
        )
    }

    /// Creates a new texture from a slice of data, encoded in one of Tetra's supported
    /// file formats (except for TGA).
    ///
    /// This is useful in combination with `include_bytes`, as it allows you to include
    /// your textures directly in the binary.
    ///
    /// The format will be determined based on the 'magic bytes' at the beginning of the
    /// data. This should be reasonably reliable, but a `from_data_with_format` function
    /// might have to be added later. Note that TGA files do not have recognizable magic
    /// bytes, so this function will not recognize them.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the underlying graphics API encounters an error.
    /// * `TetraError::InvalidTexture` will be returned if the texture data was invalid.
    pub fn from_file_data(ctx: &mut Context, data: &[u8]) -> Result<Texture> {
        let image = image::load_from_memory(data)
            .map_err(TetraError::InvalidTexture)?
            .to_rgba();

        let (width, height) = image.dimensions();

        Texture::from_rgba(
            ctx,
            width as i32,
            height as i32,
            image.into_raw().as_slice(),
        )
    }

    /// Creates a new texture from a slice of RGBA pixel data.
    ///
    /// This is useful if you wish to create a texture at runtime.
    ///
    /// Note that this method requires you to provide enough data to fill the texture.
    /// If you provide too much data, it will be truncated.
    ///
    /// # Errors
    ///
    /// * `TetraError::NotEnoughData` will be returned if not enough data is provided to fill
    /// the texture. This is to prevent the graphics API from trying to read uninitialized memory.
    pub fn from_rgba(ctx: &mut Context, width: i32, height: i32, data: &[u8]) -> Result<Texture> {
        Texture::with_device(
            &mut ctx.device,
            width,
            height,
            data,
            ctx.graphics.default_filter_mode,
        )
    }

    pub(crate) fn with_device(
        device: &mut GraphicsDevice,
        width: i32,
        height: i32,
        data: &[u8],
        filter_mode: FilterMode,
    ) -> Result<Texture> {
        let expected = (width * height * 4) as usize;
        let actual = data.len();

        if expected > actual {
            return Err(TetraError::NotEnoughData { expected, actual });
        }

        let handle = device.new_texture(width, height)?;

        device.set_texture_data(&handle, &data, 0, 0, width, height);
        device.set_texture_filter_mode(&handle, filter_mode);

        Ok(Texture {
            data: Rc::new(TextureSharedData {
                handle,
                filter_mode: Cell::new(FilterMode::Linear),
            }),
        })
    }

    pub(crate) fn with_device_empty(
        device: &mut GraphicsDevice,
        width: i32,
        height: i32,
        filter_mode: FilterMode,
    ) -> Result<Texture> {
        let handle = device.new_texture(width, height)?;
        device.set_texture_filter_mode(&handle, filter_mode);

        Ok(Texture {
            data: Rc::new(TextureSharedData {
                handle,
                filter_mode: Cell::new(filter_mode),
            }),
        })
    }

    /// Returns the width of the texture.
    pub fn width(&self) -> i32 {
        self.data.handle.width()
    }

    /// Returns the height of the texture.
    pub fn height(&self) -> i32 {
        self.data.handle.height()
    }

    /// Returns the size of the canvas.
    pub fn size(&self) -> (i32, i32) {
        (self.data.handle.width(), self.data.handle.height())
    }

    /// Returns the filter mode being used by the texture.
    pub fn filter_mode(&self) -> FilterMode {
        self.data.filter_mode.get()
    }

    /// Sets the filter mode that should be used by the texture.
    pub fn set_filter_mode(&mut self, ctx: &mut Context, filter_mode: FilterMode) {
        ctx.device
            .set_texture_filter_mode(&self.data.handle, filter_mode);

        self.data.filter_mode.set(filter_mode);
    }
}

impl Drawable for Texture {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let params = params.into();

        let texture_width = self.width() as f32;
        let texture_height = self.height() as f32;

        let (u, v, clip_width, clip_height) = match params.clip {
            Some(clip) => (clip.x, clip.y, clip.width, clip.height),
            None => (0.0, 0.0, texture_width, texture_height),
        };

        let x1 = 0.0;
        let y1 = 0.0;
        let x2 = clip_width;
        let y2 = clip_height;

        let u1 = u / texture_width;
        let v1 = v / texture_height;
        let u2 = (u + clip_width) / texture_width;
        let v2 = (v + clip_height) / texture_height;

        graphics::set_texture(ctx, self);
        graphics::push_quad(ctx, x1, y1, x2, y2, u1, v1, u2, v2, &params);
    }
}

/// Filtering algorithms that can be used when scaling an image.
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
