//! Functions and types relating to screen scaling.

use crate::graphics::Rectangle;

/// Defines the different ways that a game's screen can be scaled.
#[derive(Copy, Clone)]
pub enum ScreenScaling {
    /// The game will always be displayed at its native resolution, with no scaling applied.
    /// If the window is bigger than the native resolution, letterboxing will be applied.
    /// If the window is smaller than the native resolution, it will be cropped.
    None,

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

    /// The screen will resize to match the size of the window. More of the scene will be shown on
    /// bigger windows, and less of the scene will be shown on smaller windows.
    ///
    /// If the scaling mode changes, the internal resolution will return to its original value.
    Resize,
}

impl ScreenScaling {
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
            ScreenScaling::None => {
                let screen_x = (window_width - internal_width) / 2;
                let screen_y = (window_height - internal_height) / 2;

                Rectangle::new(
                    screen_x as f32,
                    screen_y as f32,
                    internal_width as f32,
                    internal_height as f32,
                )
            }
            ScreenScaling::Stretch | ScreenScaling::Resize => {
                Rectangle::new(0.0, 0.0, window_width as f32, window_height as f32)
            }
            ScreenScaling::ShowAll => {
                let scale_x = f_window_width / f_internal_width;
                let scale_y = f_window_height / f_internal_height;
                let scale_factor = scale_x.min(scale_y);

                let screen_width = (f_internal_width * scale_factor).ceil();
                let screen_height = (f_internal_height * scale_factor).ceil();
                let screen_x = ((f_window_width - screen_width) / 2.0).ceil();
                let screen_y = ((f_window_height - screen_height) / 2.0).ceil();

                Rectangle::new(screen_x, screen_y, screen_width, screen_height)
            }
            ScreenScaling::ShowAllPixelPerfect => {
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
            ScreenScaling::Crop => {
                let scale_x = f_window_width / f_internal_width;
                let scale_y = f_window_height / f_internal_height;
                let scale_factor = scale_x.max(scale_y);

                let screen_width = (f_internal_width * scale_factor).ceil();
                let screen_height = (f_internal_height * scale_factor).ceil();
                let screen_x = ((f_window_width - screen_width) / 2.0).ceil();
                let screen_y = ((f_window_height - screen_height) / 2.0).ceil();

                Rectangle::new(screen_x, screen_y, screen_width, screen_height)
            }
            ScreenScaling::CropPixelPerfect => {
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
