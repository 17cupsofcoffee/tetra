//! Functions and types relating to textures.

use std::path::Path;
use std::rc::Rc;

use image::{self, DynamicImage};

use crate::error::Result;
use crate::graphics::opengl::{GLTexture, TextureFormat};
use crate::graphics::{self, DrawParams, Drawable, Rectangle};
use crate::Context;

/// Texture data.
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
        let image = image::open(path)?;
        Texture::load(ctx, image)
    }

    /// Creates a new texture from a slice of binary data.
    /// 
    /// This is useful in combination with `include_bytes`, as it allows you to include
    /// your textures directly in the binary.
    ///
    /// The format will be determined based on the 'magic bytes' at the beginning of the
    /// data. This should be reasonably reliable, but a `from_data_with_format` function
    /// might have to be added later.
    ///
    /// # Errors
    ///
    /// If the image data was invalid, a `TetraError::Image` will be returned.
    pub fn from_data(ctx: &mut Context, data: &[u8]) -> Result<Texture> {
        let image = image::load_from_memory(data)?;
        Texture::load(ctx, image)
    }

    pub(crate) fn load(ctx: &mut Context, image: DynamicImage) -> Result<Texture> {
        let rgba_image = image.to_rgba();
        let (width, height) = rgba_image.dimensions();

        let texture = ctx
            .gl
            .new_texture(width as i32, height as i32, TextureFormat::Rgba);

        ctx.gl.set_texture_data(
            &texture,
            &rgba_image,
            0,
            0,
            width as i32,
            height as i32,
            TextureFormat::Rgba,
        );

        Ok(Texture::from_handle(texture))
    }

    pub(crate) fn from_handle(handle: GLTexture) -> Texture {
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
