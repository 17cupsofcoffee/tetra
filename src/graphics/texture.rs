//! Functions and types relating to textures.

use std::path::Path;
use std::rc::Rc;

use image;

use crate::error::{Result, TetraError};
use crate::graphics::opengl::{GLDevice, GLTexture, TextureFormat};
use crate::graphics::{self, DrawParams, Drawable, Rectangle};
use crate::Context;

/// A 2D texture, held in GPU memory.
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
    pub(crate) handle: Rc<GLTexture>,
}

impl Texture {
    /// Creates a new texture from the given file.
    ///
    /// The format will be determined based on the file extension.
    ///
    /// # Errors
    ///
    /// If the file could not be read, a `TetraError::Io` will be returned.
    ///
    /// If the image data was invalid, a `TetraError::Image` will be returned.
    pub fn new<P>(ctx: &mut Context, path: P) -> Result<Texture>
    where
        P: AsRef<Path>,
    {
        let image = image::open(path)?.to_rgba();
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
    /// If the image data was invalid, a `TetraError::Image` will be returned.
    pub fn from_file_data(ctx: &mut Context, data: &[u8]) -> Result<Texture> {
        let image = image::load_from_memory(data)?.to_rgba();
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
    /// If not enough data is provided to fill the texture, a `TetraError::NotEnoughData`
    /// will be returned. This is to prevent OpenGL from reading uninitialized memory.
    pub fn from_rgba(ctx: &mut Context, width: i32, height: i32, data: &[u8]) -> Result<Texture> {
        Texture::with_device(&mut ctx.gl, width, height, data)
    }

    #[deprecated(
        since = "0.2.13",
        note = "Renamed to `from_file_data` to disambiguate from other image data formats (e.g. RGBA)."
    )]
    #[allow(missing_docs)]
    #[inline]
    pub fn from_data(ctx: &mut Context, data: &[u8]) -> Result<Texture> {
        Texture::from_file_data(ctx, data)
    }

    pub(crate) fn with_device(
        device: &mut GLDevice,
        width: i32,
        height: i32,
        data: &[u8],
    ) -> Result<Texture> {
        let expected = (width * height * 4) as usize;
        let actual = data.len();

        if expected > actual {
            return Err(TetraError::NotEnoughData { expected, actual });
        }

        let handle = device.new_texture(width, height, TextureFormat::Rgba);

        device.set_texture_data(&handle, &data, 0, 0, width, height, TextureFormat::Rgba);

        Ok(Texture {
            handle: Rc::new(handle),
        })
    }

    pub(crate) fn with_device_empty(device: &mut GLDevice, width: i32, height: i32) -> Texture {
        let handle = device.new_texture(width, height, TextureFormat::Rgba);

        Texture {
            handle: Rc::new(handle),
        }
    }

    /// Returns the width of the texture.
    pub fn width(&self) -> i32 {
        self.handle.width()
    }

    /// Returns the height of the texture.
    pub fn height(&self) -> i32 {
        self.handle.height()
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
        let clip = params
            .clip
            .unwrap_or_else(|| Rectangle::new(0.0, 0.0, texture_width, texture_height));

        let x1 = 0.0;
        let y1 = 0.0;
        let x2 = clip.width;
        let y2 = clip.height;

        let u1 = clip.x / texture_width;
        let v1 = clip.y / texture_height;
        let u2 = (clip.x + clip.width) / texture_width;
        let v2 = (clip.y + clip.height) / texture_height;

        graphics::set_texture(ctx, self);
        graphics::push_quad(ctx, x1, y1, x2, y2, u1, v1, u2, v2, &params);
    }
}
