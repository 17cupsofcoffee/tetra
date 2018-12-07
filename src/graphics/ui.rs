//! Functions and types relating to user interfaces.

use glm::Vec3;

use graphics::{self, DrawParams, Drawable, Rectangle, Texture};
use Context;

/// A panel made up of nine slices of an image. Useful for panels with borders.
///
/// Note that `NineSlice` does not currently support the `clip` `DrawParam`.
pub struct NineSlice {
    texture: Texture,
    width: f32,
    height: f32,
    fill_rect: Rectangle,
}

impl NineSlice {
    /// Creates a new NineSlice from the given texture.
    ///
    /// The `fill_rect` is used to determine how to slice the texture - it should be set
    /// to the region of the texture that represents the center of the panel.
    pub fn new(texture: Texture, width: f32, height: f32, fill_rect: Rectangle) -> NineSlice {
        NineSlice {
            texture,
            width,
            height,
            fill_rect,
        }
    }
}

impl Drawable for NineSlice {
    fn draw<T: Into<DrawParams>>(&self, ctx: &mut Context, params: T) {
        graphics::set_texture(ctx, &self.texture);

        assert!(
            ctx.graphics.sprite_count < ctx.graphics.capacity,
            "Renderer is full"
        );

        let params = params.into();
        let transform = params.build_matrix();

        let texture_width = self.texture.handle.width() as f32;
        let texture_height = self.texture.handle.height() as f32;

        let pos1 = transform * Vec3::new(0.0, 0.0, 1.0);
        let pos2 = transform * Vec3::new(self.fill_rect.x, self.fill_rect.y, 1.0);
        let pos3 = transform * Vec3::new(
            self.width - self.fill_rect.x,
            self.height - self.fill_rect.y,
            1.0,
        );
        let pos4 = transform * Vec3::new(self.width, self.height, 1.0);

        let u1 = 0.0;
        let v1 = 0.0;
        let u2 = self.fill_rect.x / texture_width;
        let v2 = self.fill_rect.y / texture_height;
        let u3 = (self.fill_rect.x + self.fill_rect.width) / texture_width;
        let v3 = (self.fill_rect.y + self.fill_rect.height) / texture_height;
        let u4 = 1.0;
        let v4 = 1.0;

        // Top left
        graphics::push_quad(
            ctx,
            pos1.x,
            pos1.y,
            pos2.x,
            pos2.y,
            u1,
            v1,
            u2,
            v2,
            params.color,
        );

        // Top
        graphics::push_quad(
            ctx,
            pos2.x,
            pos1.y,
            pos3.x,
            pos2.y,
            u2,
            v1,
            u3,
            v2,
            params.color,
        );

        // Top right
        graphics::push_quad(
            ctx,
            pos3.x,
            pos1.y,
            pos4.x,
            pos2.y,
            u3,
            v1,
            u4,
            v2,
            params.color,
        );

        // Left
        graphics::push_quad(
            ctx,
            pos1.x,
            pos2.y,
            pos2.x,
            pos3.y,
            u1,
            v2,
            u2,
            v3,
            params.color,
        );

        // Center
        graphics::push_quad(
            ctx,
            pos2.x,
            pos2.y,
            pos3.x,
            pos3.y,
            u2,
            v2,
            u3,
            v3,
            params.color,
        );

        // Right
        graphics::push_quad(
            ctx,
            pos3.x,
            pos2.y,
            pos4.x,
            pos3.y,
            u3,
            v2,
            u4,
            v3,
            params.color,
        );

        // Bottom left
        graphics::push_quad(
            ctx,
            pos1.x,
            pos3.y,
            pos2.x,
            pos4.y,
            u1,
            v3,
            u2,
            v4,
            params.color,
        );

        // Bottom
        graphics::push_quad(
            ctx,
            pos2.x,
            pos3.y,
            pos3.x,
            pos4.y,
            u2,
            v3,
            u3,
            v4,
            params.color,
        );

        // Bottom right
        graphics::push_quad(
            ctx,
            pos3.x,
            pos3.y,
            pos4.x,
            pos4.y,
            u3,
            v3,
            u4,
            v4,
            params.color,
        );
    }
}
