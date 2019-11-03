//! Functions and types relating to handling the player's input.
//!
//! # Gamepads
//!
//! When accessing gamepad state, you specify which gamepad you're interested in via a 'gamepad index'.
//! The first gamepad connected to the system has index 0, the second has index 1, and so on.
//!
//! If a controller is disconnected, the next controller to be connected will take its index - otherwise,
//! a new one will be allocated. This behaviour might be made smarter in future versions.

mod gamepad;
mod keyboard;
mod mouse;

use hashbrown::HashSet;

use crate::math::Vec2;
use crate::Context;

pub use gamepad::*;
pub use keyboard::*;
pub use mouse::*;

pub(crate) struct InputContext {
    current_key_state: HashSet<Key>,
    previous_key_state: HashSet<Key>,
    current_text_input: Option<String>,

    current_mouse_state: HashSet<MouseButton>,
    previous_mouse_state: HashSet<MouseButton>,
    mouse_position: Vec2<f32>,

    pads: Vec<Option<GamepadState>>,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            current_key_state: HashSet::new(),
            previous_key_state: HashSet::new(),
            current_text_input: None,

            current_mouse_state: HashSet::new(),
            previous_mouse_state: HashSet::new(),
            mouse_position: Vec2::zero(),

            pads: Vec::new(),
        }
    }
}

pub(crate) fn cleanup_after_state_update(ctx: &mut Context) {
    ctx.input.previous_key_state = ctx.input.current_key_state.clone();
    ctx.input.previous_mouse_state = ctx.input.current_mouse_state.clone();
    ctx.input.current_text_input = None;

    for slot in &mut ctx.input.pads {
        if let Some(pad) = slot {
            pad.previous_button_state = pad.current_button_state.clone();
        }
    }
}

/// Returns the text that the user entered this tick.
/// This will match the user's keyboard and OS settings.
pub fn get_text_input(ctx: &Context) -> Option<&str> {
    ctx.input.current_text_input.as_ref().map(String::as_str)
}

// TODO: Remove this once WASM text input support is added
#[cfg_attr(target_arch = "wasm32", allow(unused))]
pub(crate) fn set_text_input(ctx: &mut Context, text: Option<String>) {
    ctx.input.current_text_input = text;
}
