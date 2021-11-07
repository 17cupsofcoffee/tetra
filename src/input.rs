//! Functions and types relating to handling the player's input.
//!
//! # Gamepads
//!
//! When accessing gamepad state, you specify which gamepad you're interested in via a 'gamepad ID'.
//! The first gamepad connected to the system has ID 0, the second has ID 1, and so on.
//!
//! If a controller is disconnected, the next controller to be connected will take its ID - otherwise,
//! a new one will be allocated. This means that if you unplug a controller and then plug it back in,
//! it should retain its existing ID. This behaviour might be made smarter in future versions.
//!
//! # Examples
//!
//! The [`keyboard`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/keyboard.rs)
//! example demonstrates how to handle keyboard input.
//!
//! The [`mouse`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/mouse.rs)
//! example demonstrates how to handle mouse input.
//!
//! The [`gamepad`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/gamepad.rs)
//! example demonstrates how to handle gamepad input.
//!
//! The [`text_input`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/text_input.rs)
//! example demonstrates how to handle text entry.

mod gamepad;
mod keyboard;
mod mouse;

use hashbrown::HashSet;

use crate::math::Vec2;
use crate::{Context, Result};

pub use gamepad::*;
pub use keyboard::*;
pub use mouse::*;

pub(crate) struct InputContext {
    keys_down: HashSet<Key>,
    keys_pressed: HashSet<Key>,
    keys_released: HashSet<Key>,

    key_modifier_state: KeyModifierState,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    mouse_position: Vec2<f32>,
    mouse_wheel_movement: Vec2<i32>,

    current_text_input: Option<String>,

    pads: Vec<Option<GamepadState>>,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),

            key_modifier_state: KeyModifierState::default(),

            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            mouse_position: Vec2::zero(),
            mouse_wheel_movement: Vec2::zero(),

            current_text_input: None,

            pads: Vec::new(),
        }
    }
}

pub(crate) fn clear(ctx: &mut Context) {
    ctx.input.keys_pressed.clear();
    ctx.input.keys_released.clear();
    ctx.input.mouse_buttons_pressed.clear();
    ctx.input.mouse_buttons_released.clear();
    ctx.input.mouse_wheel_movement = Vec2::zero();

    ctx.input.current_text_input = None;

    for pad in ctx.input.pads.iter_mut().flatten() {
        pad.buttons_pressed.clear();
        pad.buttons_released.clear();
    }
}

/// Returns the text that the user entered since the last update.
/// This will match the user's keyboard and OS settings.
pub fn get_text_input(ctx: &Context) -> Option<&str> {
    ctx.input.current_text_input.as_deref()
}

/// Gets the text currently stored in the system's clipboard.
///
/// # Errors
///
/// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be
/// returned if the text could not be retrieved from the clipboard.
pub fn get_clipboard_text(ctx: &Context) -> Result<String> {
    ctx.window.get_clipboard_text()
}

/// Sets the contents of the system's clipboard.
///
/// # Errors
///
/// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be
/// returned if the clipboard could not be modified.
pub fn set_clipboard_text(ctx: &Context, text: &str) -> Result {
    ctx.window.set_clipboard_text(text)
}

pub(crate) fn push_text_input(ctx: &mut Context, text: &str) {
    match &mut ctx.input.current_text_input {
        Some(existing) => existing.push_str(text),
        x @ None => *x = Some(text.to_string()),
    }
}
