//! Functions and types relating to the game window.

use crate::{Context, Result};

/// Quits the game, if it is currently running.
///
/// Note that currently, quitting the game does not take effect until the end of the current
/// cycle of the game loop. This will probably change later.
pub fn quit(ctx: &mut Context) {
    ctx.running = false;
}

/// Gets the current title of the window.
pub fn get_title(ctx: &Context) -> &str {
    ctx.platform.get_window_title()
}

/// Sets the title of the window.
pub fn set_title<S>(ctx: &mut Context, title: S)
where
    S: AsRef<str>,
{
    ctx.platform.set_window_title(title);
}

/// Gets the width of the window.
pub fn get_width(ctx: &Context) -> i32 {
    ctx.platform.get_window_size().0
}

/// Sets the width of the window.
pub fn set_width(ctx: &mut Context, width: i32) {
    set_size(ctx, width, get_height(ctx));
}

/// Gets the height of the window.
pub fn get_height(ctx: &Context) -> i32 {
    ctx.platform.get_window_size().1
}

/// Sets the height of the window.
pub fn set_height(ctx: &mut Context, height: i32) {
    set_size(ctx, get_width(ctx), height);
}

/// Gets the size of the window.
pub fn get_size(ctx: &Context) -> (i32, i32) {
    ctx.platform.get_window_size()
}

/// Sets the size of the window.
pub fn set_size(ctx: &mut Context, width: i32, height: i32) {
    ctx.platform.set_window_size(width, height);
}

/// Enables fullscreen if it is currently disabled, or vice-versa.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Sdl`.
pub fn toggle_fullscreen(ctx: &mut Context) -> Result {
    ctx.platform.toggle_fullscreen()
}

/// Enables fullscreen.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Sdl`.
pub fn enable_fullscreen(ctx: &mut Context) -> Result {
    ctx.platform.enable_fullscreen()
}

/// Disables fullscreen.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Sdl`.
pub fn disable_fullscreen(ctx: &mut Context) -> Result {
    ctx.platform.disable_fullscreen()
}

/// Returns whether or not the window is currently in fullscreen mode.
pub fn is_fullscreen(ctx: &Context) -> bool {
    ctx.platform.is_fullscreen()
}

/// Makes the mouse cursor visible.
pub fn show_mouse(ctx: &mut Context) {
    ctx.platform.set_mouse_visible(true);
}

/// Hides the mouse cursor.
pub fn hide_mouse(ctx: &mut Context) {
    ctx.platform.set_mouse_visible(false);
}

/// Returns whether or not the mouse cursor is currently visible.
pub fn is_mouse_visible(ctx: &Context) -> bool {
    ctx.platform.is_mouse_visible()
}
