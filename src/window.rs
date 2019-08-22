//! Functions and types relating to the game window.

use crate::platform;
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
    platform::get_window_title(ctx)
}

/// Sets the title of the window.
pub fn set_title<S>(ctx: &mut Context, title: S)
where
    S: AsRef<str>,
{
    platform::set_window_title(ctx, title)
}

/// Gets the width of the window.
pub fn get_width(ctx: &Context) -> i32 {
    platform::get_window_width(ctx)
}

/// Sets the width of the window.
pub fn set_width(ctx: &mut Context, width: i32) {
    set_size(ctx, width, platform::get_window_height(ctx));
}

/// Gets the height of the window.
pub fn get_height(ctx: &Context) -> i32 {
    platform::get_window_height(ctx)
}

/// Sets the height of the window.
pub fn set_height(ctx: &mut Context, height: i32) {
    set_size(ctx, platform::get_window_width(ctx), height);
}

/// Gets the size of the window.
pub fn get_size(ctx: &Context) -> (i32, i32) {
    platform::get_window_size(ctx)
}

/// Sets the size of the window.
pub fn set_size(ctx: &mut Context, width: i32, height: i32) {
    platform::set_window_size(ctx, width, height);
}

/// Enables fullscreen if it is currently disabled, or vice-versa.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Platform`.
pub fn toggle_fullscreen(ctx: &mut Context) -> Result {
    platform::toggle_fullscreen(ctx)
}

/// Enables fullscreen.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Platform`.
pub fn enable_fullscreen(ctx: &mut Context) -> Result {
    platform::enable_fullscreen(ctx)
}

/// Disables fullscreen.
///
/// # Errors
///
/// If the application's fullscreen state could not be changed, this function
/// will return a `TetraError::Platform`.
pub fn disable_fullscreen(ctx: &mut Context) -> Result {
    platform::disable_fullscreen(ctx)
}

/// Returns whether or not the window is currently in fullscreen mode.
pub fn is_fullscreen(ctx: &Context) -> bool {
    platform::is_fullscreen(ctx)
}

/// Makes the mouse cursor visible.
pub fn show_mouse(ctx: &mut Context) {
    platform::set_mouse_visible(ctx, true);
}

/// Hides the mouse cursor.
pub fn hide_mouse(ctx: &mut Context) {
    platform::set_mouse_visible(ctx, false);
}

/// Returns whether or not the mouse cursor is currently visible.
pub fn is_mouse_visible(ctx: &Context) -> bool {
    platform::is_mouse_visible(ctx)
}
