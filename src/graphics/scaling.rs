//! Functions and types relating to screen scaling.

use crate::error::Result;
use crate::graphics::{self, Canvas, DrawParams, Drawable, Rectangle};
use crate::window;
use crate::Context;

#[derive(Debug)]
pub struct ScreenScaler {
    canvas: Canvas,
    mode: ScalingMode,
    screen_rect: Rectangle,
}

impl ScreenScaler {
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
            mode.get_screen_rect(inner_width, inner_height, outer_width, outer_height);

        Ok(ScreenScaler {
            canvas,
            mode,
            screen_rect,
        })
    }

    pub fn match_window(
        ctx: &mut Context,
        inner_width: i32,
        inner_height: i32,
        mode: ScalingMode,
    ) -> Result<ScreenScaler> {
        ScreenScaler::new(
            ctx,
            inner_width,
            inner_height,
            window::get_width(ctx),
            window::get_height(ctx),
            mode,
        )
    }

    pub fn inner_width(&self) -> i32 {
        self.canvas().width()
    }

    pub fn inner_height(&self) -> i32 {
        self.canvas().height()
    }

    pub fn inner_size(&self) -> (i32, i32) {
        (self.inner_width(), self.inner_height())
    }
    
    pub fn outer_width(&self) -> i32 {
        self.screen_rect.width as i32
    }

    pub fn set_outer_width(&mut self, outer_width: i32) {
        self.set_outer_size(outer_width, self.outer_height());
    }

    pub fn outer_height(&self) -> i32 {
        self.screen_rect.height as i32
    }

    pub fn set_outer_height(&mut self, outer_height: i32) {
        self.set_outer_size(self.outer_width(), outer_height);
    }

    pub fn outer_size(&self) -> (i32, i32) {
        (self.outer_width(), self.outer_height())
    }

    pub fn set_outer_size(&mut self, outer_width: i32, outer_height: i32) {
        self.screen_rect = self.mode.get_screen_rect(
            self.inner_width(),
            self.inner_height(),
            outer_width,
            outer_height,
        );
    }

    pub fn sync_with_window(&mut self, ctx: &Context) {
        self.set_outer_size(window::get_width(ctx), window::get_height(ctx));
    }

    pub fn canvas(&self) -> &Canvas {
        &self.canvas
    }

    pub fn mode(&self) -> ScalingMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: ScalingMode) {
        self.mode = mode;
        self.screen_rect = self.mode.get_screen_rect(
            self.inner_width(),
            self.inner_height(),
            self.outer_width(),
            self.outer_height(),
        );
    }
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

/// Defines the different ways that a game's screen can be scaled.
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

impl ScalingMode {
    pub(crate) fn get_screen_rect(
        self,
        internal_width: i32,
        internal_height: i32,
        window_width: i32,
        window_height: i32,
    ) -> Rectangle {
        let f_internal_width = internal_width as f32;
        let f_internal_height = internal_height as f32;
        let f_window_width = window_width as f32;
        let f_window_height = window_height as f32;

        let internal_aspect_ratio = f_internal_width / f_internal_height;
        let screen_aspect_ratio = f_window_width / f_window_height;

        match self {
            ScalingMode::Fixed => {
                let screen_x = (window_width - internal_width) / 2;
                let screen_y = (window_height - internal_height) / 2;

                Rectangle::new(
                    screen_x as f32,
                    screen_y as f32,
                    internal_width as f32,
                    internal_height as f32,
                )
            }
            ScalingMode::Stretch => {
                Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32)
            }
            ScalingMode::ShowAll => {
                let scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                    f_window_width / f_internal_width
                } else {
                    f_window_height / f_internal_height
                };

                let screen_width = (f_internal_width * scale_factor).ceil();
                let screen_height = (f_internal_height * scale_factor).ceil();
                let screen_x = ((f_window_width - screen_width) / 2.0).ceil();
                let screen_y = ((f_window_height - screen_height) / 2.0).ceil();

                Rectangle::new(screen_x, screen_y, screen_width, screen_height)
            }
            ScalingMode::ShowAllPixelPerfect => {
                let mut scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                    window_width / internal_width
                } else {
                    window_height / internal_height
                };

                if scale_factor == 0 {
                    scale_factor = 1;
                }

                let screen_width = internal_width * scale_factor;
                let screen_height = internal_height * scale_factor;
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
                let scale_x = f_window_width / f_internal_width;
                let scale_y = f_window_height / f_internal_height;
                let scale_factor = scale_x.max(scale_y);

                let screen_width = (f_internal_width * scale_factor).ceil();
                let screen_height = (f_internal_height * scale_factor).ceil();
                let screen_x = ((f_window_width - screen_width) / 2.0).ceil();
                let screen_y = ((f_window_height - screen_height) / 2.0).ceil();

                Rectangle::new(screen_x, screen_y, screen_width, screen_height)
            }
            ScalingMode::CropPixelPerfect => {
                let mut scale_factor = if internal_aspect_ratio > screen_aspect_ratio {
                    f_window_height / f_internal_height
                } else {
                    f_window_width / f_internal_width
                }
                .ceil() as i32;

                if scale_factor == 0 {
                    scale_factor = 1;
                }

                let screen_width = internal_width * scale_factor;
                let screen_height = internal_height * scale_factor;
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
}
