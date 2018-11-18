use std::path::Path;
use std::rc::Rc;

use image;

use error::{Result, TetraError};
use graphics::opengl::GLTexture;
use graphics::{self, DrawParams, Drawable, Rectangle};
use Context;

#[derive(Clone, PartialEq)]
pub struct Texture {
    pub(crate) handle: Rc<GLTexture>,
    pub width: i32,
    pub height: i32,
}

impl Texture {
    pub fn new<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Result<Texture> {
        let image = image::open(path).map_err(TetraError::Image)?.to_rgba();
        let (width, height) = image.dimensions();

        let texture = ctx.gl.new_texture(width as i32, height as i32);
        ctx.gl
            .set_texture_data(&texture, &image, 0, 0, width as i32, height as i32);

        Ok(Texture {
            handle: Rc::new(texture),
            width: width as i32,
            height: height as i32,
        })
    }
}

impl Drawable for Texture {
    fn draw<T: Into<DrawParams>>(&self, ctx: &mut Context, params: T) {
        graphics::set_texture(ctx, self);

        assert!(
            ctx.render_state.sprite_count < ctx.render_state.capacity,
            "Renderer is full"
        );

        let params = params.into();

        let texture_width = self.width as f32;
        let texture_height = self.height as f32;
        let clip = params
            .clip
            .unwrap_or_else(|| Rectangle::new(0.0, 0.0, texture_width, texture_height));

        // TODO: I feel like there must be a cleaner way of determining the winding order...
        // TODO: We could probably use GLM to do this with vector math, too

        let (x1, x2, u1, u2) = if params.scale.x >= 0.0 {
            (
                params.position.x - params.origin.x,
                params.position.x - params.origin.x + (clip.width * params.scale.x),
                clip.x / texture_width,
                (clip.x + clip.width) / texture_width,
            )
        } else {
            (
                params.position.x + params.origin.x + (clip.width * params.scale.x),
                params.position.x + params.origin.x,
                (clip.x + clip.width) / texture_width,
                clip.x / texture_width,
            )
        };

        let (y1, y2, v1, v2) = if params.scale.y >= 0.0 {
            (
                params.position.y - params.origin.y,
                params.position.y - params.origin.y + (clip.height * params.scale.y),
                clip.y / texture_height,
                (clip.y + clip.height) / texture_height,
            )
        } else {
            (
                params.position.y + params.origin.y + (clip.height * params.scale.y),
                params.position.y + params.origin.y,
                (clip.y + clip.height) / texture_height,
                clip.y / texture_height,
            )
        };

        graphics::push_vertex(ctx, x1, y1, u1, v1, params.color);
        graphics::push_vertex(ctx, x1, y2, u1, v2, params.color);
        graphics::push_vertex(ctx, x2, y2, u2, v2, params.color);
        graphics::push_vertex(ctx, x2, y1, u2, v1, params.color);

        ctx.render_state.sprite_count += 1;
    }
}
