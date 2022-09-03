//! Functions and types relating to animations.

use std::time::Duration;

use crate::graphics::texture::Texture;
use crate::graphics::{DrawParams, Rectangle};
use crate::time;
use crate::Context;

/// An animation, cycling between regions of a texture at a regular interval.
///
/// Calling [`advance`](Self::advance) or [`advance`](Self::advance_by) within [`State::draw`](crate::State::draw)
/// will drive the animation, switching the texture region once the specified
/// time has passed.
///
/// # Examples
///
/// The [`animation`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/animation.rs)
/// example demonstrates basic usage of an `Animation` with a spritesheet.
///
/// The [`animation_controller`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/animation_controller.rs)
/// example demonstrates how multiple `Animation`s can be combined using a
/// simple state machine.
#[derive(Debug, Clone)]
pub struct Animation {
    texture: Texture,
    frames: Vec<Rectangle>,
    frame_length: Duration,

    current_frame: usize,
    timer: Duration,
    repeating: bool,
}

impl Animation {
    /// Creates a new looping animation.
    pub fn new(texture: Texture, frames: Vec<Rectangle>, frame_length: Duration) -> Animation {
        Animation {
            texture,
            frames,
            frame_length,

            current_frame: 0,
            timer: Duration::from_secs(0),
            repeating: true,
        }
    }

    /// Creates a new animation that does not repeat once all of the frames have been displayed.
    pub fn once(texture: Texture, frames: Vec<Rectangle>, frame_length: Duration) -> Animation {
        Animation {
            texture,
            frames,
            frame_length,

            current_frame: 0,
            timer: Duration::from_secs(0),
            repeating: false,
        }
    }

    /// Draws the current frame to the screen (or to a canvas, if one is enabled).
    pub fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let frame = self.frames[self.current_frame];

        self.texture.draw_region(ctx, frame, params);
    }

    /// Advances the animation's timer, switching the texture region if required.
    ///
    /// This method uses the current [delta time](crate::time::get_delta_time)
    /// to calculate how much time has passed.
    pub fn advance(&mut self, ctx: &Context) {
        self.advance_by(time::get_delta_time(ctx));
    }

    /// Advances the animation's timer by a specified amount, switching the texture
    /// region if required.
    ///
    /// If the specified duration is longer than the frame length, frames will be
    /// skipped.
    pub fn advance_by(&mut self, duration: Duration) {
        self.timer += duration;

        let frames_remaining = self.has_frames_remaining();

        if frames_remaining || self.repeating {
            while self.timer >= self.frame_length {
                self.current_frame = (self.current_frame + 1) % self.frames.len();
                self.timer -= self.frame_length;
            }
        } else if self.timer > self.frame_length {
            self.timer = self.frame_length;
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
    /// for the new texture, call [`set_frames`](Self::set_frames).
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

    /// Gets whether or not the animation is currently set to repeat when it reaches the end
    /// of the frames.
    pub fn repeating(&self) -> bool {
        self.repeating
    }

    /// Sets whether or not the animation should repeat when it reaches the end of the frames.
    pub fn set_repeating(&mut self, repeating: bool) {
        self.repeating = repeating;
    }

    /// Gets the index of the frame that is currently being displayed.
    ///
    /// This index is zero-based, and can be used in combination with the [`frames`](Self::frames)
    /// method in order to track the progress of the animation.
    pub fn current_frame_index(&self) -> usize {
        self.current_frame
    }

    /// Sets which frame of the animation should be displayed.
    ///
    /// Usually you will want to control the animation by calling [`advance`](Self::advance)
    /// or [`advance`](Self::advance_by), but this method can be useful for more
    /// fine-grained control.
    ///
    /// # Panics
    ///
    /// The index is zero-based, and must be within the bounds of the animation's
    /// [`frames`](Self::frames), otherwise this method will panic.
    pub fn set_current_frame_index(&mut self, index: usize) {
        // Without this check, the code would panic in `Drawable::draw` because `self.frames[self.current_frame]`
        // is invalid, but the developer would have no clue where it was set.
        assert!(index < self.frames.len());

        self.current_frame = index;
    }

    /// Gets the duration that the current frame has been visible.
    ///
    /// This can be used in combination with the [`frame_length`](Self::frame_length) method
    /// in order to track the progress of the animation.
    pub fn current_frame_time(&self) -> Duration {
        self.timer
    }

    /// Sets the duration that the current frame has been visible.
    ///
    /// Usually you will want to control the animation by calling [`advance`](Self::advance)
    /// or [`advance`](Self::advance_by),but this method can be useful for more
    /// fine-grained control.
    ///
    /// The animation will not advance past the end of the current frame until the next call
    /// to [`advance`](Self::advance) or [`advance`](Self::advance_by). If a value is given
    /// that is larger than [`frame_length`](Self::frame_length), this animation may
    /// skip frames.
    pub fn set_current_frame_time(&mut self, duration: Duration) {
        self.timer = duration;
    }

    /// Returns true if this animation will no longer advance.
    ///
    /// Will always be false for repeating animations.
    pub fn is_finished(&self) -> bool {
        !self.repeating && !self.has_frames_remaining()
    }

    /// Returns true if there are any frames remaining in the current cycle.
    pub fn has_frames_remaining(&self) -> bool {
        self.current_frame < self.frames.len() - 1
    }
}
