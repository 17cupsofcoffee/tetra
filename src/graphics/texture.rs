//! Functions and types relating to textures.

use std::path::Path;
use std::rc::Rc;

use glm::{Vec2, Vec3};
use image;

use error::{Result, TetraError};
use graphics::opengl::GLTexture;
use graphics::{self, DrawParams, Drawable, Rectangle};
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

        let texture = ctx.gl.new_texture(width as i32, height as i32);
        ctx.gl
            .set_texture_data(&texture, &image, 0, 0, width as i32, height as i32);

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
        graphics::set_texture(ctx, self);

        assert!(
            ctx.graphics.sprite_count < ctx.graphics.capacity,
            "Renderer is full"
        );

        let params = params.into();

        let texture_width = self.handle.width() as f32;
        let texture_height = self.handle.height() as f32;
        let clip = params
            .clip
            .unwrap_or_else(|| Rectangle::new(0.0, 0.0, texture_width, texture_height));

        let transform = params.build_matrix();

        let pos1 = (transform * Vec3::new(0.0, 0.0, 1.0)).xy();
        let pos2 = (transform * Vec3::new(clip.width, clip.height, 1.0)).xy();

        let tex1 = Vec2::new(clip.x / texture_width, clip.y / texture_height);
        let tex2 = Vec2::new(
            (clip.x + clip.width) / texture_width,
            (clip.y + clip.height) / texture_height,
        );

        graphics::push_quad(ctx, pos1, pos2, tex1, tex2, params.color);
    }
}
