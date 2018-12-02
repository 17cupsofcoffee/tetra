//! Functions and types relating to handling user input (e.g. keyboards, mice, gamepads).

use fnv::FnvHashSet;
use glm::Vec2;

use graphics;
use Context;

/// Represents a key on the player's keyboard.
pub use sdl2::keyboard::Keycode as Key;

pub(crate) struct InputContext {
    pub(crate) current_key_state: FnvHashSet<Key>,
    pub(crate) previous_key_state: FnvHashSet<Key>,
    pub(crate) mouse_position: Vec2,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            current_key_state: FnvHashSet::default(),
            previous_key_state: FnvHashSet::default(),
            mouse_position: Vec2::zeros(),
        }
    }
}

/// Returns true if the specified key is currently down.
pub fn is_key_down(ctx: &Context, key: Key) -> bool {
    ctx.input.current_key_state.contains(&key)
}

/// Returns true if the specified key is currently up.
pub fn is_key_up(ctx: &Context, key: Key) -> bool {
    !ctx.input.current_key_state.contains(&key)
}

/// Returns true if the specified key was pressed this tick.
pub fn is_key_pressed(ctx: &Context, key: Key) -> bool {
    !ctx.input.previous_key_state.contains(&key) && ctx.input.current_key_state.contains(&key)
}

/// Returns true if the specified key was released this tick.
pub fn is_key_released(ctx: &Context, key: Key) -> bool {
    ctx.input.previous_key_state.contains(&key) && !ctx.input.current_key_state.contains(&key)
}

/// Get the X co-ordinate of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_x(ctx: &Context) -> f32 {
    (ctx.input.mouse_position.x / graphics::get_window_width(ctx) as f32)
        * graphics::get_width(ctx) as f32
}

/// Get the Y co-ordinate of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_y(ctx: &Context) -> f32 {
    (ctx.input.mouse_position.y / graphics::get_window_height(ctx) as f32)
        * graphics::get_height(ctx) as f32
}

/// Get the position of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_position(ctx: &Context) -> Vec2 {
    Vec2::new(get_mouse_x(ctx), get_mouse_y(ctx))
}
