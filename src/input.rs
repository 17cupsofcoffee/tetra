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

mod gamepad;
mod keyboard;
mod mouse;

use hashbrown::{HashMap, HashSet};

use crate::math::Vec2;
use crate::Context;

pub use gamepad::*;
pub use keyboard::*;
pub use mouse::*;

pub(crate) struct InputContext {
    keys_down: HashSet<Key>,
    keys_pressed: HashSet<Key>,
    keys_released: HashSet<Key>,

    mouse_buttons_down: HashSet<MouseButton>,
    mouse_buttons_pressed: HashSet<MouseButton>,
    mouse_buttons_released: HashSet<MouseButton>,
    mouse_position: Vec2<f32>,

    current_text_input: Option<String>,

    pads: Vec<Option<GamepadState>>,
    platform_id_mappings: HashMap<i32, usize>,
}

impl InputContext {
    pub(crate) fn new() -> InputContext {
        InputContext {
            keys_down: HashSet::new(),
            keys_pressed: HashSet::new(),
            keys_released: HashSet::new(),

            mouse_buttons_down: HashSet::new(),
            mouse_buttons_pressed: HashSet::new(),
            mouse_buttons_released: HashSet::new(),
            mouse_position: Vec2::zero(),

            current_text_input: None,

            pads: Vec::new(),
            platform_id_mappings: HashMap::new(),
        }
    }
}

pub(crate) fn clear(ctx: &mut Context) {
    ctx.input.keys_pressed.clear();
    ctx.input.keys_released.clear();
    ctx.input.mouse_buttons_pressed.clear();
    ctx.input.mouse_buttons_released.clear();

    ctx.input.current_text_input = None;

    for slot in &mut ctx.input.pads {
        if let Some(pad) = slot {
            pad.buttons_pressed.clear();
            pad.buttons_released.clear();
        }
    }
}

/// Returns the text that the user entered since the last update.
/// This will match the user's keyboard and OS settings.
pub fn get_text_input(ctx: &Context) -> Option<&str> {
    ctx.input.current_text_input.as_ref().map(String::as_str)
}

pub(crate) fn push_text_input(ctx: &mut Context, text: &str) {
    match &mut ctx.input.current_text_input {
        Some(existing) => existing.push_str(text),
        x @ None => *x = Some(text.to_string()),
    }
}
