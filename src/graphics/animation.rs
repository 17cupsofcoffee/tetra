//! Functions and types relating to animations.

use graphics::texture::Texture;
use graphics::{DrawParams, Drawable, Rectangle};
use Context;

/// An animaton, cycling between regions of a texture at a regular interval.
///
/// As the rendering speed of the game is not fixed, use the `tick` method in your
/// `update` handler to progress the animation.
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

    /// Set new frames for this animation, while keeping the old texture and frame length. This will reset the current animation.
    pub fn set_frames(&mut self, new_frames: Vec<Rectangle>) {
        self.frames = new_frames;
        self.current_frame = 0;
        self.timer = 0;
    }

    /// Set the new frame length for this animation. This will make the animation run at the new length right away.
    /// If you want to reset the animation to 0, call `restart_animation`
    pub fn set_frame_length(&mut self, new_frame_length: i32) {
        self.frame_length = new_frame_length;
    }

    /// Will restart the current animation from the beginning.
    pub fn restart_animation(&mut self) {
        self.current_frame = 0;
        self.timer = 0;
    }
}

impl Drawable for Animation {
    fn draw<T: Into<DrawParams>>(&self, ctx: &mut Context, params: T) {
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
