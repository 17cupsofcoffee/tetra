//! Functions and types relating to animations.

use crate::graphics::texture::Texture;
use crate::graphics::{DrawParams, Drawable, Rectangle};
use crate::Context;

/// An animaton, cycling between regions of a texture at a regular interval.
///
/// As the rendering speed of the game is not fixed, use the `tick` method in your
/// `update` handler to progress the animation.
#[derive(Debug, Clone)]
pub struct Animation {
    texture: Texture,
    frames: Vec<Rectangle>,
    frame_length: i32,

    current_frame: usize,
    timer: i32,
}

impl Animation {
    /// Creates a new animation.
    pub fn new(texture: Texture, frames: Vec<Rectangle>, frame_length: i32) -> Animation {
        Animation {
            texture,
            frames,
            frame_length,

            current_frame: 0,
            timer: 0,
        }
    }

    /// Advances the animation's timer, switching the texture region if required.
    pub fn tick(&mut self) {
        self.timer += 1;
        if self.timer >= self.frame_length {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.timer = 0;
        }
    }

    /// Restarts the animation from the first frame.
    pub fn restart(&mut self) {
        self.current_frame = 0;
        self.timer = 0;
    }

    /// Gets the texture currently being used by the animation.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    /// Sets the texture that will be used by the animation.
    ///
    /// This method will not change the frame definitions or current state of the animation,
    /// so it can be used for e.g. swapping spritesheets. If you need to change the slicing
    /// for the new texture, call `set_frames`.
    pub fn set_texture(&mut self, texture: Texture) {
        self.texture = texture;
    }

    /// Gets the sections of the texture being displayed for each frame of the animation.
    pub fn frames(&self) -> &[Rectangle] {
        &self.frames
    }

    /// Sets the sections of the texture being displayed for each frame of the animation.
    ///
    /// This method will reset the animation back to frame zero.
    pub fn set_frames(&mut self, new_frames: Vec<Rectangle>) {
        self.frames = new_frames;

        self.restart();
    }

    /// Gets the number of ticks that each frame of the animation lasts for.
    pub fn frame_length(&self) -> i32 {
        self.frame_length
    }

    /// Sets the number of ticks that each frame of the animation lasts for.
    pub fn set_frame_length(&mut self, new_frame_length: i32) {
        self.frame_length = new_frame_length;
    }
}

impl Drawable for Animation {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let frame_clip = self.frames[self.current_frame];

        let mut params = params.into();

        params.clip = match params.clip {
            Some(mut clip) => {
                clip.x += frame_clip.x;
                clip.y += frame_clip.y;
                clip.width += frame_clip.width;
                clip.height += frame_clip.height;

                Some(clip)
            }
            None => Some(frame_clip),
        };

        self.texture.draw(ctx, params)
    }
}
