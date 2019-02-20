//! Functions and types relating to the game window.

use sdl2::video::FullscreenType;

use crate::graphics::{self, ScreenScaling};
use crate::{Context, Result, TetraError};

/// Quits the game, if it is currently running.
///
/// Note that currently, quitting the game does not take effect until the end of the current
/// cycle of the game loop. This will probably change later.
pub fn quit(ctx: &mut Context) {
    ctx.running = false;
}

/// Gets the current title of the window.
pub fn get_title(ctx: &Context) -> &str {
    ctx.window.title()
}

/// Sets the title of the window.
pub fn set_title(ctx: &mut Context, title: &str) {
    ctx.window.set_title(title).unwrap();
}

/// Gets the width of the window.
pub fn get_width(ctx: &Context) -> i32 {
    ctx.window_width
}

/// Sets the width of the window.
pub fn set_width(ctx: &mut Context, width: i32) {
    set_size_ex(ctx, width, ctx.window_height, false);
}

/// Gets the height of the window.
pub fn get_height(ctx: &Context) -> i32 {
    ctx.window_height
}

/// Sets the height of the window.
pub fn set_height(ctx: &mut Context, height: i32) {
    set_size_ex(ctx, ctx.window_width, height, false);
}

/// Gets the size of the window.
pub fn get_size(ctx: &Context) -> (i32, i32) {
    (ctx.window_width, ctx.window_height)
}

/// Sets the size of the window.
pub fn set_size(ctx: &mut Context, width: i32, height: i32) {
    set_size_ex(ctx, width, height, false);
}

pub(crate) fn set_size_ex(ctx: &mut Context, width: i32, height: i32, from_sdl: bool) {
    ctx.window_width = width;
    ctx.window_height = height;

    graphics::set_window_projection(ctx, width, height);

    if let ScreenScaling::Resize = graphics::get_scaling(ctx) {
        graphics::set_backbuffer_size(ctx, width, height);
    }

    graphics::update_screen_rect(ctx);

    if !from_sdl {
        ctx.window.set_size(width as u32, height as u32).unwrap();
    }
}

/// Enables fullscreen if it is currently disabled, or vice-versa.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Sdl`.
pub fn toggle_fullscreen(ctx: &mut Context) -> Result {
    if ctx.fullscreen {
        disable_fullscreen(ctx)
    } else {
        enable_fullscreen(ctx)
    }
}

/// Enables fullscreen.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Sdl`.
pub fn enable_fullscreen(ctx: &mut Context) -> Result {
    if !ctx.fullscreen {
        ctx.window
            .display_mode()
            .and_then(|m| {
                set_size_ex(ctx, m.w, m.h, false);
                ctx.window.set_fullscreen(FullscreenType::Desktop)
            })
            .map(|_| ())
            .map_err(TetraError::Sdl)
    } else {
        Ok(())
    }
}

/// Disables fullscreen.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Sdl`.
pub fn disable_fullscreen(ctx: &mut Context) -> Result {
    if ctx.fullscreen {
        ctx.window
            .set_fullscreen(FullscreenType::Off)
            .map(|_| {
                let size = ctx.window.drawable_size();
                set_size_ex(ctx, size.0 as i32, size.1 as i32, false);
            })
            .map_err(TetraError::Sdl)
    } else {
        Ok(())
    }
}

/// Returns whether or not the window is currently in fullscreen mode.
pub fn is_fullscreen(ctx: &Context) -> bool {
    ctx.fullscreen
}

/// Makes the mouse cursor visible.
pub fn show_mouse(ctx: &mut Context) {
    ctx.sdl.mouse().show_cursor(true);
}

/// Hides the mouse cursor.
pub fn hide_mouse(ctx: &mut Context) {
    ctx.sdl.mouse().show_cursor(false);
}

/// Returns whether or not the mouse cursor is currently visible.
pub fn is_mouse_visible(ctx: &mut Context) {
    ctx.sdl.mouse().is_cursor_showing();
}
