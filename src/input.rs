//! Functions and types relating to handling user input (e.g. keyboards, mice, gamepads).

use fnv::FnvHashSet;
use glm::Vec2;

use graphics;
use Context;

/// Represents a key on the player's keyboard.
pub use sdl2::keyboard::Keycode as Key;

/// Represents a button on the player's mouse.
pub use sdl2::mouse::MouseButton;

pub(crate) struct InputContext {
    pub(crate) current_key_state: FnvHashSet<Key>,
    pub(crate) previous_key_state: FnvHashSet<Key>,
    pub(crate) current_mouse_state: FnvHashSet<MouseButton>,
    pub(crate) previous_mouse_state: FnvHashSet<MouseButton>,
    pub(crate) mouse_position: Vec2,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            current_key_state: FnvHashSet::default(),
            previous_key_state: FnvHashSet::default(),
            current_mouse_state: FnvHashSet::default(),
            previous_mouse_state: FnvHashSet::default(),
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

/// Returns a list of keys that were pressed this tick. This can be used for text input.
pub fn get_key_strokes<'a>(ctx: &'a Context) -> impl Iterator<Item = Key> + 'a {
    ctx.input
        .current_key_state
        .iter()
        .cloned()
        .filter(move |k| !ctx.input.previous_key_state.contains(&k))
}

/// Returns true if the specified mouse button is currently down.
pub fn is_mouse_button_down(ctx: &Context, button: MouseButton) -> bool {
    ctx.input.current_mouse_state.contains(&button)
}

/// Returns true if the specified mouse button is currently up.
pub fn is_mouse_button_up(ctx: &Context, button: MouseButton) -> bool {
    !ctx.input.current_mouse_state.contains(&button)
}

/// Returns true if the specified mouse button was pressed this tick.
pub fn is_mouse_button_pressed(ctx: &Context, button: MouseButton) -> bool {
    !ctx.input.previous_mouse_state.contains(&button)
        && ctx.input.current_mouse_state.contains(&button)
}

/// Returns true if the specified mouse button was released this tick.
pub fn is_mouse_button_released(ctx: &Context, button: MouseButton) -> bool {
    ctx.input.previous_mouse_state.contains(&button)
        && !ctx.input.current_mouse_state.contains(&button)
}

/// Get the X co-ordinate of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_x(ctx: &Context) -> f32 {
    let internal_width = graphics::get_width(ctx) as f32;
    let letterbox = graphics::get_letterbox(ctx);

    ((ctx.input.mouse_position.x - letterbox.x) / letterbox.width) * internal_width
}

/// Get the Y co-ordinate of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_y(ctx: &Context) -> f32 {
    let internal_height = graphics::get_height(ctx) as f32;
    let letterbox = graphics::get_letterbox(ctx);

    ((ctx.input.mouse_position.y - letterbox.y) / letterbox.height) * internal_height
}

/// Get the position of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_position(ctx: &Context) -> Vec2 {
    Vec2::new(get_mouse_x(ctx), get_mouse_y(ctx))
}
