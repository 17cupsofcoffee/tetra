//! Functions and types relating to screen scaling.

use crate::error::Result;
use crate::graphics::{self, Canvas, DrawParams, Rectangle};
use crate::input;
use crate::math::Vec2;
use crate::window;
use crate::Context;

/// A wrapper for a [`Canvas`] that handles scaling the image to fit the screen.
///
/// # Examples
///
/// The [`scaling`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/scaling.rs)
/// example demonstrates how to use a `ScreenScaler` with each of the different
/// scaling algorithms.
#[derive(Debug)]
pub struct ScreenScaler {
    canvas: Canvas,
    mode: ScalingMode,
    screen_rect: Rectangle,
    outer_width: i32,
    outer_height: i32,
}

impl ScreenScaler {
    /// Returns a new `ScreenScaler`, with the specified inner and outer width and height.
    /// The mode will determine how the image is scaled to fit the screen.
    pub fn new(
        ctx: &mut Context,
        inner_width: i32,
        inner_height: i32,
        outer_width: i32,
        outer_height: i32,
        mode: ScalingMode,
    ) -> Result<ScreenScaler> {
        let canvas = Canvas::new(ctx, inner_width, inner_height)?;
        let screen_rect =
            get_screen_rect(mode, inner_width, inner_height, outer_width, outer_height);

        Ok(ScreenScaler {
            canvas,
            mode,
            screen_rect,
            outer_width,
            outer_height,
        })
    }

    /// Returns a new `ScreenScaler`, with the specified inner width and height, and the outer
    /// size set to the current dimensions of the window.
    pub fn with_window_size(
        ctx: &mut Context,
        inner_width: i32,
        inner_height: i32,
        mode: ScalingMode,
    ) -> Result<ScreenScaler> {
        let (outer_width, outer_height) = window::get_size(ctx);

        ScreenScaler::new(
            ctx,
            inner_width,
            inner_height,
            outer_width,
            outer_height,
            mode,
        )
    }

    /// Draws the scaled image to the screen.
    pub fn draw(&self, ctx: &mut Context) {
        graphics::set_texture(ctx, &self.canvas.texture);

        graphics::push_quad(
            ctx,
            self.screen_rect.x,
            self.screen_rect.y,
            self.screen_rect.x + self.screen_rect.width,
            self.screen_rect.y + self.screen_rect.height,
            0.0,
            0.0,
            1.0,
            1.0,
            &DrawParams::new(),
        );
    }

    /// Updates the scaler's outer size (i.e. the size of the box that the screen will be scaled to
    /// fit within).
    pub fn set_outer_size(&mut self, outer_width: i32, outer_height: i32) {
        if outer_width != self.outer_width || outer_height != self.outer_height {
            self.outer_width = outer_width;
            self.outer_height = outer_height;

            self.screen_rect = get_screen_rect(
                self.mode,
                self.canvas().width(),
                self.canvas().height(),
                outer_width,
                outer_height,
            );
        }
    }

    /// Returns a reference to the canvas that is being scaled.
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Returns the current scaling mode.
    pub fn mode(&self) -> ScalingMode {
        self.mode
    }

    /// Sets the scaling mode that should be used.
    pub fn set_mode(&mut self, mode: ScalingMode) {
        self.mode = mode;
        self.screen_rect = get_screen_rect(
            self.mode,
            self.canvas().width(),
            self.canvas().height(),
            self.outer_width,
            self.outer_height,
        );
    }

    /// Converts a point from window co-ordinates to scaled screen co-ordinates.
    pub fn project(&self, position: Vec2<f32>) -> Vec2<f32> {
        let (width, height) = self.canvas().size();

        Vec2::new(
            project_impl(
                position.x,
                self.screen_rect.x,
                self.screen_rect.width,
                width as f32,
            ),
            project_impl(
                position.y,
                self.screen_rect.y,
                self.screen_rect.height,
                height as f32,
            ),
        )
    }

    /// Converts a point from scaled screen co-ordinates to window co-ordinates.
    pub fn unproject(&self, position: Vec2<f32>) -> Vec2<f32> {
        let (width, height) = self.canvas().size();

        Vec2::new(
            unproject_impl(
                position.x,
                self.screen_rect.x,
                self.screen_rect.width,
                width as f32,
            ),
            unproject_impl(
                position.y,
                self.screen_rect.y,
                self.screen_rect.height,
                height as f32,
            ),
        )
    }

    /// Returns the position of the mouse in scaled screen co-ordinates.
    ///
    /// This is a shortcut for calling [`.project(input::get_mouse_position(ctx))`](Self::project).
    pub fn mouse_position(&self, ctx: &Context) -> Vec2<f32> {
        self.project(input::get_mouse_position(ctx))
    }

    /// Returns the X co-ordinate of the mouse in scaled screen co-ordinates.
    ///
    /// This is a shortcut for calling [`project(input::get_mouse_position(ctx)).x`](Self::project).
    pub fn mouse_x(&self, ctx: &Context) -> f32 {
        let width = self.canvas().width();

        project_impl(
            input::get_mouse_x(ctx),
            self.screen_rect.x,
            self.screen_rect.width,
            width as f32,
        )
    }

    /// Returns the Y co-ordinate of the mouse in scaled screen co-ordinates.
    ///
    /// This is a shortcut for calling [`project(input::get_mouse_position(ctx)).y`](Self::project).
    pub fn mouse_y(&self, ctx: &Context) -> f32 {
        let height = self.canvas().height();

        project_impl(
            input::get_mouse_y(ctx),
            self.screen_rect.y,
            self.screen_rect.height,
            height as f32,
        )
    }
}

fn project_impl(window_pos: f32, rect_pos: f32, rect_size: f32, real_size: f32) -> f32 {
    (real_size * (window_pos - rect_pos)) / rect_size
}

fn unproject_impl(screen_pos: f32, rect_pos: f32, rect_size: f32, real_size: f32) -> f32 {
    rect_pos + ((rect_size * screen_pos) / real_size)
}

/// Algorithms that can be used to scale the game's screen.
#[non_exhaustive]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ScalingMode {
    /// The game will always be displayed at its native resolution, with no scaling applied.
    /// If the window is bigger than the native resolution, letterboxing will be applied.
    /// If the window is smaller than the native resolution, it will be cropped.
    Fixed,

    /// The screen will be stretched to fill the window, without trying to preserve the original
    /// aspect ratio. Distortion/stretching/squashing may occur.
    Stretch,

    /// The entire screen will be displayed as large as possible while maintaining the original
    /// aspect ratio. Letterboxing may occur.
    ShowAll,

    /// Works the same as ShowAll, but will only scale by integer values.
    ShowAllPixelPerfect,

    /// The screen will fill the entire window, maintaining the original aspect ratio but
    /// potentially being cropped.
    Crop,

    /// Works the same as Crop, but will only scale by integer values.
    CropPixelPerfect,
}

/// Converts a screen's dimensions into a rectangle that is scaled to fit in the given bounds.
///
/// This function may be useful if you want to use Tetra's scaling algorithms, but
/// the built-in [`ScreenScaler`] abstraction does not fit your needs.
pub fn get_screen_rect(
    mode: ScalingMode,
    inner_width: i32,
    inner_height: i32,
    outer_width: i32,
    outer_height: i32,
) -> Rectangle {
    let f_inner_width = inner_width as f32;
    let f_inner_height = inner_height as f32;
    let f_outer_width = outer_width as f32;
    let f_outer_height = outer_height as f32;

    let internal_aspect_ratio = f_inner_width / f_inner_height;
    let screen_aspect_ratio = f_outer_width / f_outer_height;

    match mode {
        ScalingMode::Fixed => {
            let screen_x = (outer_width - inner_width) / 2;
            let screen_y = (outer_height - inner_height) / 2;

            Rectangle::new(
                screen_x as f32,
                screen_y as f32,
                inner_width as f32,
                inner_height as f32,
            )
        }
        ScalingMode::Stretch => Rectangle::new(0.0, 0.0, outer_width as f32, outer_height as f32),
        ScalingMode::ShowAll => {
            let scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                f_outer_width / f_inner_width
            } else {
                f_outer_height / f_inner_height
            };

            let screen_width = (f_inner_width * scale_factor).ceil();
            let screen_height = (f_inner_height * scale_factor).ceil();
            let screen_x = ((f_outer_width - screen_width) / 2.0).ceil();
            let screen_y = ((f_outer_height - screen_height) / 2.0).ceil();

            Rectangle::new(screen_x, screen_y, screen_width, screen_height)
        }
        ScalingMode::ShowAllPixelPerfect => {
            let mut scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                outer_width / inner_width
            } else {
                outer_height / inner_height
            };

            if scale_factor == 0 {
                scale_factor = 1;
            }

            let screen_width = inner_width * scale_factor;
            let screen_height = inner_height * scale_factor;
            let screen_x = (outer_width - screen_width) / 2;
            let screen_y = (outer_height - screen_height) / 2;

            Rectangle::new(
                screen_x as f32,
                screen_y as f32,
                screen_width as f32,
                screen_height as f32,
            )
        }
        ScalingMode::Crop => {
            let scale_x = f_outer_width / f_inner_width;
            let scale_y = f_outer_height / f_inner_height;
            let scale_factor = scale_x.max(scale_y);

            let screen_width = (f_inner_width * scale_factor).ceil();
            let screen_height = (f_inner_height * scale_factor).ceil();
            let screen_x = ((f_outer_width - screen_width) / 2.0).ceil();
            let screen_y = ((f_outer_height - screen_height) / 2.0).ceil();

            Rectangle::new(screen_x, screen_y, screen_width, screen_height)
        }
        ScalingMode::CropPixelPerfect => {
            let mut scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                f_outer_height / f_inner_height
            } else {
                f_outer_width / f_inner_width
            }
            .ceil() as i32;

            if scale_factor == 0 {
                scale_factor = 1;
            }

            let screen_width = inner_width * scale_factor;
            let screen_height = inner_height * scale_factor;
            let screen_x = (outer_width - screen_width) / 2;
            let screen_y = (outer_height - screen_height) / 2;

            Rectangle::new(
                screen_x as f32,
                screen_y as f32,
                screen_width as f32,
                screen_height as f32,
            )
        }
    }
}
