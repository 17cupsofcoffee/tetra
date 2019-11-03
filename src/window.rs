//! Functions and types relating to the game window.

use crate::platform;
use crate::{Context, Result};

/// Quits the game, if it is currently running.
///
/// Note that quitting the game does not take effect until the end of the current
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
///
/// # Errors
///
/// * `TetraError::FailedToChangeDisplayMode` will be returned if the game was unable to
/// change the window size.
pub fn set_width(ctx: &mut Context, width: i32) -> Result {
    set_size(ctx, width, platform::get_window_height(ctx))
}

/// Gets the height of the window.
pub fn get_height(ctx: &Context) -> i32 {
    platform::get_window_height(ctx)
}

/// Sets the height of the window.
///
/// # Errors
///
/// * `TetraError::FailedToChangeDisplayMode` will be returned if the game was unable to
/// change the window size.
pub fn set_height(ctx: &mut Context, height: i32) -> Result {
    set_size(ctx, platform::get_window_width(ctx), height)
}

/// Gets the size of the window.
pub fn get_size(ctx: &Context) -> (i32, i32) {
    platform::get_window_size(ctx)
}

/// Sets the size of the window.
///
/// # Errors
///
/// * `TetraError::FailedToChangeDisplayMode` will be returned if the game was unable to
/// change the window size.
pub fn set_size(ctx: &mut Context, width: i32, height: i32) -> Result {
    platform::set_window_size(ctx, width, height)
}

/// Sets whether the window should be vsynced.
///
/// # Errors
///
/// * `TetraError::FailedToChangeDisplayMode` will be returned if the game was unable to
/// change vsync mode.
pub fn set_vsync(ctx: &mut Context, vsync: bool) -> Result {
    platform::set_vsync(ctx, vsync)
}

/// Returns whethere or not vsync is enabled.
pub fn is_vsync_enabled(ctx: &Context) -> bool {
    platform::is_vsync_enabled(ctx)
}

/// Sets whether the window should be in fullscreen mode.
///
/// # Errors
///
/// * `TetraError::FailedToChangeDisplayMode` will be returned if the game was unable to
/// enter or exit fullscreen.
pub fn set_fullscreen(ctx: &mut Context, fullscreen: bool) -> Result {
    platform::set_fullscreen(ctx, fullscreen)
}

/// Returns whether or not the window is currently in fullscreen mode.
pub fn is_fullscreen(ctx: &Context) -> bool {
    platform::is_fullscreen(ctx)
}

/// Sets whether or not the mouse cursor should be visible.
///
/// # Errors
///
/// * `TetraError::PlatformError` will be returned if the cursor state was inaccessible.
pub fn set_mouse_visible(ctx: &mut Context) -> Result {
    platform::set_mouse_visible(ctx, true)
}

/// Returns whether or not the mouse cursor is currently visible.
pub fn is_mouse_visible(ctx: &Context) -> bool {
    platform::is_mouse_visible(ctx)
}
