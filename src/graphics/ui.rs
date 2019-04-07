//! Functions and types relating to user interfaces.

use crate::graphics::{self, DrawParams, Drawable, Rectangle, Texture};
use crate::Context;

/// A panel made up of nine slices of an image. Useful for panels with borders.
///
/// Note that `NineSlice` does not currently support the `clip` `DrawParam`.
#[derive(Debug, Clone)]
pub struct NineSlice {
    texture: Texture,
    width: f32,
    height: f32,
    fill_rect: Rectangle,
}

impl NineSlice {
    /// Creates a new panel from the given texture.
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

    /// Gets the underlying texture for the panel.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Sets the underlying texture for the panel.
    ///
    /// This will not adjust the way that the texture is sliced, so you may need to also call
    /// `set_fill_rect`.
    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }

    /// Gets the width of the panel.
    pub fn width(&self) -> f32 {
        self.width
    }

    /// Sets the width of the panel.
    pub fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    /// Gets the height of the panel.
    pub fn height(&self) -> f32 {
        self.height
    }

    /// Sets the height of the panel.
    pub fn set_height(&mut self, height: f32) {
        self.height = height;
    }

    /// Gets the size of the panel.
    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    /// Sets the size of the panel.
    pub fn set_size(&mut self, width: f32, height: f32) {
        self.width = width;
        self.height = height;
    }

    /// Gets the section of the texture that is being used to fill the center of the panel.
    pub fn fill_rect(&self) -> &Rectangle {
        &self.fill_rect
    }

    /// Sets the section of the texture that should fill the center of the panel.
    pub fn set_fill_rect(&mut self, fill_rect: Rectangle) {
        self.fill_rect = fill_rect;
    }
}

impl Drawable for NineSlice {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let params = params.into();

        let texture_width = self.texture.width() as f32;
        let texture_height = self.texture.height() as f32;

        let x1 = 0.0;
        let y1 = 0.0;
        let x2 = self.fill_rect.x;
        let y2 = self.fill_rect.y;
        let x3 = self.width - self.fill_rect.x;
        let y3 = self.height - self.fill_rect.y;
        let x4 = self.width;
        let y4 = self.height;

        let u1 = 0.0;
        let v1 = 0.0;
        let u2 = self.fill_rect.x / texture_width;
        let v2 = self.fill_rect.y / texture_height;
        let u3 = (self.fill_rect.x + self.fill_rect.width) / texture_width;
        let v3 = (self.fill_rect.y + self.fill_rect.height) / texture_height;
        let u4 = 1.0;
        let v4 = 1.0;

        graphics::set_texture(ctx, &self.texture);

        // Top left
        graphics::push_quad(ctx, x1, y1, x2, y2, u1, v1, u2, v2, &params);

        // Top
        graphics::push_quad(ctx, x2, y1, x3, y2, u2, v1, u3, v2, &params);

        // Top right
        graphics::push_quad(ctx, x3, y1, x4, y2, u3, v1, u4, v2, &params);

        // Left
        graphics::push_quad(ctx, x1, y2, x2, y3, u1, v2, u2, v3, &params);

        // Center
        graphics::push_quad(ctx, x2, y2, x3, y3, u2, v2, u3, v3, &params);

        // Right
        graphics::push_quad(ctx, x3, y2, x4, y3, u3, v2, u4, v3, &params);

        // Bottom left
        graphics::push_quad(ctx, x1, y3, x2, y4, u1, v3, u2, v4, &params);

        // Bottom
        graphics::push_quad(ctx, x2, y3, x3, y4, u2, v3, u3, v4, &params);

        // Bottom right
        graphics::push_quad(ctx, x3, y3, x4, y4, u3, v3, u4, v4, &params);
    }
}
