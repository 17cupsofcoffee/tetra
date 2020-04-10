//! Functions and types relating to animations.

use std::time::Duration;

use crate::graphics::texture::Texture;
use crate::graphics::{DrawParams, Drawable, Rectangle};
use crate::time;
use crate::Context;

/// An animation, cycling between regions of a texture at a regular interval.
///
/// Calling `advance` within your `draw` method will drive the animation, switching the texture
/// region once the specified time has passed.
#[derive(Debug, Clone)]
pub struct Animation {
    texture: Texture,
    frames: Vec<Rectangle>,
    frame_length: Duration,

    current_frame: usize,
    timer: Duration,
}

impl Animation {
    /// Creates a new animation.
    pub fn new(texture: Texture, frames: Vec<Rectangle>, frame_length: Duration) -> Animation {
        Animation {
            texture,
            frames,
            frame_length,

            current_frame: 0,
            timer: Duration::from_secs(0),
        }
    }

    /// Advances the animation's timer, switching the texture region if required.
    pub fn advance(&mut self, ctx: &Context) {
        self.advance_by(time::get_delta_time(ctx));
    }

    /// Advances the animation's timer by a specified amount, switching the texture region if required.
    pub fn advance_by(&mut self, duration: Duration) {
        self.timer += duration;

        while self.timer >= self.frame_length {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.timer -= self.frame_length;
        }
    }

    /// Restarts the animation from the first frame.
    pub fn restart(&mut self) {
        self.current_frame = 0;
        self.timer = Duration::from_secs(0);
    }

    /// Returns a reference to the texture currently being used by the animation.
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

    /// Gets the amount of time that each frame of the animation lasts for.
    pub fn frame_length(&self) -> Duration {
        self.frame_length
    }

    /// Sets the amount of time that each frame of the animation lasts for.
    pub fn set_frame_length(&mut self, new_frame_length: Duration) {
        self.frame_length = new_frame_length;
    }

    /// Get the current frame that is active. This can be used in combination with [frames](#method.frames) to track the progress of this animation.
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Set the current frame that is active.  This can be used for fine-tuned animation manipulation.
    ///
    /// The given value should be a valid index in the [frames](#method.frames) list, otherwise this animation will panic.
    pub fn set_current_frame_index(&mut self, new_frame_index: usize) {
        // Without this check, the code would panic in `Drawable::draw` because `self.frames[self.current_frame]` is invalid,
        // but the developer would have no clue where it was set.
        debug_assert!(self.frames.get(new_frame_index).is_some());

        self.current_frame = new_frame_index;
    }

    /// Get the duration that this frame has been visible. This can be used in combination with [frame_length](#method.frame_length) to track the progress of this animation.
    pub fn current_frame_time(&self) -> Duration {
        self.timer
    }

    /// Set the duration that the current frame has been visible. This can be used for fine-tuned animation manipulation.
    ///
    /// The animation will not update until the next call to [advance](#method.advance) or [advance_by](#method.advance_by). If a value is given that is larger than [frame_length](#method.frame_length), this animation may skip frames.
    pub fn set_current_frame_time(&mut self, new_frame_time: Duration) {
        self.timer = new_frame_time;
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
