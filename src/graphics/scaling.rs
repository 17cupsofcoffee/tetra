//! Functions and types relating to screen scaling.

use crate::error::Result;
use crate::graphics::{self, Canvas, DrawParams, Drawable, Rectangle};
use crate::input;
use crate::math::Vec2;
use crate::window;
use crate::Context;

/// A wrapper for a `Canvas` that handles scaling the image to fit the screen.
///
/// # Examples
///
/// ```rust
/// # use tetra::{Context, State};
/// # use tetra::graphics::{self, Color};
/// # use tetra::graphics::scaling::{ScreenScaler, ScalingMode};
/// # use tetra::math::Vec2;
/// #
/// struct GameState {
///     scaler: ScreenScaler,
/// }
///
/// impl GameState {
///     fn new(ctx: &mut Context) -> tetra::Result<GameState> {
///         Ok(GameState {
///             scaler: ScreenScaler::new(ctx, 128, 128, ScalingMode::ShowAllPixelPerfect)?
///         })
///     }
/// }
///
/// impl State for GameState {
///     fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
///         graphics::set_canvas(ctx, self.scaler.canvas());
///
///         // Draw your scene here...
///
///         graphics::reset_canvas(ctx);
///         graphics::clear(ctx, Color::BLACK);
///         graphics::draw(ctx, &self.scaler, Vec2::new(0.0, 0.0));
///
///         Ok(())
///     }
///
///     fn size_changed(&mut self, _ctx: &mut Context, width: i32, height: i32) -> tetra::Result {
///         // Ensure your scaler is kept aware of screen size changes!
///         self.scaler.set_window_size(width, height);
///
///         Ok(())
///     }
/// }
/// ```
#[derive(Debug)]
pub struct ScreenScaler {
    canvas: Canvas,
    mode: ScalingMode,
    screen_rect: Rectangle,
    window_width: i32,
    window_height: i32,
}

impl ScreenScaler {
    /// Returns a new `ScreenScaler`, with the specified internal width and height. The mode will
    /// determine how the image is scaled to fit the screen.
    pub fn new(
        ctx: &mut Context,
        width: i32,
        height: i32,
        mode: ScalingMode,
    ) -> Result<ScreenScaler> {
        let (window_width, window_height) = window::get_size(ctx);

        let canvas = Canvas::new(ctx, width, height)?;
        let screen_rect = get_screen_rect(mode, width, height, window_width, window_height);

        Ok(ScreenScaler {
            canvas,
            mode,
            screen_rect,
            window_width,
            window_height,
        })
    }

    /// Updates the screen's size to fit the current size of the window.
    ///
    /// If your window never changes size, you don't need to call this.
    pub fn set_window_size(&mut self, window_width: i32, window_height: i32) {
        if window_width != self.window_width || window_height != self.window_height {
            self.window_width = window_width;
            self.window_height = window_height;

            self.screen_rect = get_screen_rect(
                self.mode,
                self.canvas().width(),
                self.canvas().height(),
                window_width,
                window_height,
            );
        }
    }

    /// Returns a reference to the canvas that is being scaled.
    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    /// Returns a mutable reference to the canvas that is being scaled.
    pub fn canvas_mut(&mut self) -> &mut Canvas {
        &mut self.canvas
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
            self.window_width,
            self.window_height,
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
    /// This is a shortcut for calling `.project(input::get_mouse_position(ctx))`.
    pub fn mouse_position(&self, ctx: &Context) -> Vec2<f32> {
        self.project(input::get_mouse_position(ctx))
    }

    /// Returns the X co-ordinate of the mouse in scaled screen co-ordinates.
    ///
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx)).x`.
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
    /// This is a shortcut for calling `project(input::get_mouse_position(ctx)).y`.
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

impl Drawable for ScreenScaler {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
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
            &params.into(),
        );
    }
}

/// Algorithms that can be used to scale the game's screen.
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

/// Converts a screen's dimensions into a rectangle that is scaled to fit in the given window size.
///
/// This function may be useful if you want to use Tetra's scaling algorithms, but
/// the built-in `ScreenScaler` abstraction does not fit your needs.
pub fn get_screen_rect(
    mode: ScalingMode,
    original_width: i32,
    original_height: i32,
    window_width: i32,
    window_height: i32,
) -> Rectangle {
    let f_original_width = original_width as f32;
    let f_original_height = original_height as f32;
    let f_window_width = window_width as f32;
    let f_window_height = window_height as f32;

    let internal_aspect_ratio = f_original_width / f_original_height;
    let screen_aspect_ratio = f_window_width / f_window_height;

    match mode {
        ScalingMode::Fixed => {
            let screen_x = (window_width - original_width) / 2;
            let screen_y = (window_height - original_height) / 2;

            Rectangle::new(
                screen_x as f32,
                screen_y as f32,
                original_width as f32,
                original_height as f32,
            )
        }
        ScalingMode::Stretch => Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32),
        ScalingMode::ShowAll => {
            let scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                f_window_width / f_original_width
            } else {
                f_window_height / f_original_height
            };

            let screen_width = (f_original_width * scale_factor).ceil();
            let screen_height = (f_original_height * scale_factor).ceil();
            let screen_x = ((f_window_width - screen_width) / 2.0).ceil();
            let screen_y = ((f_window_height - screen_height) / 2.0).ceil();

            Rectangle::new(screen_x, screen_y, screen_width, screen_height)
        }
        ScalingMode::ShowAllPixelPerfect => {
            let mut scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                window_width / original_width
            } else {
                window_height / original_height
            };

            if scale_factor == 0 {
                scale_factor = 1;
            }

            let screen_width = original_width * scale_factor;
            let screen_height = original_height * scale_factor;
            let screen_x = (window_width - screen_width) / 2;
            let screen_y = (window_height - screen_height) / 2;

            Rectangle::new(
                screen_x as f32,
                screen_y as f32,
                screen_width as f32,
                screen_height as f32,
            )
        }
        ScalingMode::Crop => {
            let scale_x = f_window_width / f_original_width;
            let scale_y = f_window_height / f_original_height;
            let scale_factor = scale_x.max(scale_y);

            let screen_width = (f_original_width * scale_factor).ceil();
            let screen_height = (f_original_height * scale_factor).ceil();
            let screen_x = ((f_window_width - screen_width) / 2.0).ceil();
            let screen_y = ((f_window_height - screen_height) / 2.0).ceil();

            Rectangle::new(screen_x, screen_y, screen_width, screen_height)
        }
        ScalingMode::CropPixelPerfect => {
            let mut scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                f_window_height / f_original_height
            } else {
                f_window_width / f_original_width
            }
            .ceil() as i32;

            if scale_factor == 0 {
                scale_factor = 1;
            }

            let screen_width = original_width * scale_factor;
            let screen_height = original_height * scale_factor;
            let screen_x = (window_width - screen_width) / 2;
            let screen_y = (window_height - screen_height) / 2;

            Rectangle::new(
                screen_x as f32,
                screen_y as f32,
                screen_width as f32,
                screen_height as f32,
            )
        }
    }
}
