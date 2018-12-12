//! Functions and types relating to textures.

use std::path::Path;
use std::rc::Rc;

use image;

use error::{Result, TetraError};
use graphics::opengl::{GLTexture, TextureFormat};
use graphics::{self, ActiveShader, DrawParams, Drawable, Rectangle};
use Context;

/// Texture data.
///
/// This type acts as a lightweight handle to the associated graphics hardware data,
/// and so can be cloned with little overhead.
#[derive(Clone, PartialEq)]
pub struct Texture {
    pub(crate) handle: Rc<GLTexture>,
}

impl Texture {
    /// Creates a new texture from the given file.
    pub fn new<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Result<Texture> {
        let image = image::open(path).map_err(TetraError::Image)?.to_rgba();
        let (width, height) = image.dimensions();

        let texture = ctx
            .gl
            .new_texture(width as i32, height as i32, TextureFormat::Rgba);

        ctx.gl.set_texture_data(
            &texture,
            &image,
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
}

impl Drawable for Texture {
    fn draw<T: Into<DrawParams>>(&self, ctx: &mut Context, params: T) {
        let params = params.into();
        let transform = params.build_matrix();

        let texture_width = self.handle.width() as f32;
        let texture_height = self.handle.height() as f32;
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
        graphics::set_shader_ex(ctx, ActiveShader::Default);
        graphics::push_quad(
            ctx,
            x1,
            y1,
            x2,
            y2,
            u1,
            v1,
            u2,
            v2,
            &transform,
            params.color,
        );
    }
}
