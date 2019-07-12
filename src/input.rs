//! Functions and types relating to handling the player's input.
//!
//! # Gamepads
//!
//! When accessing gamepad state, you specify which gamepad you're interested in via a 'gamepad index'.
//! The first gamepad connected to the system has index 0, the second has index 1, and so on.
//!
//! If a controller is disconnected, the next controller to be connected will take its index - otherwise,
//! a new one will be allocated. This behaviour might be made smarter in future versions.

use hashbrown::{HashMap, HashSet};

use crate::glm::Vec2;
use crate::graphics;
use crate::Context;

// TODO: Replace these with Tetra-specific types
pub use sdl2::keyboard::Keycode as Key;
pub use sdl2::mouse::MouseButton;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum GamepadButton {
    A,
    B,
    X,
    Y,
    Up,
    Down,
    Left,
    Right,
    LeftShoulder,
    LeftTrigger,
    LeftStick,
    RightShoulder,
    RightTrigger,
    RightStick,
    Start,
    Back,
    Guide,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum GamepadAxis {
    LeftStickX,
    LeftStickY,
    LeftTrigger,
    RightStickX,
    RightStickY,
    RightTrigger,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum GamepadStick {
    LeftStick,
    RightStick,
}

pub(crate) struct GamepadState {
    platform_id: i32,
    current_button_state: HashSet<GamepadButton>,
    previous_button_state: HashSet<GamepadButton>,
    current_axis_state: HashMap<GamepadAxis, f32>,
}

impl GamepadState {
    pub(crate) fn new(platform_id: i32) -> GamepadState {
        GamepadState {
            platform_id,
            current_button_state: HashSet::new(),
            previous_button_state: HashSet::new(),
            current_axis_state: HashMap::new(),
        }
    }

    pub(crate) fn set_button_down(&mut self, btn: GamepadButton) {
        self.current_button_state.insert(btn);
    }

    pub(crate) fn set_button_up(&mut self, btn: GamepadButton) {
        self.current_button_state.remove(&btn);
    }

    pub(crate) fn set_axis_position(&mut self, axis: GamepadAxis, value: f32) {
        self.current_axis_state.insert(axis, value);
    }
}

pub(crate) struct InputContext {
    current_key_state: HashSet<Key>,
    previous_key_state: HashSet<Key>,
    current_text_input: Option<String>,

    current_mouse_state: HashSet<MouseButton>,
    previous_mouse_state: HashSet<MouseButton>,
    mouse_position: Vec2,

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
            mouse_position: Vec2::zeros(),

            pads: Vec::new(),
        }
    }
}

pub(crate) fn set_key_down(ctx: &mut Context, key: Key) {
    ctx.input.current_key_state.insert(key);
}

pub(crate) fn set_key_up(ctx: &mut Context, key: Key) {
    ctx.input.current_key_state.remove(&key);
}

pub(crate) fn set_mouse_button_down(ctx: &mut Context, btn: MouseButton) {
    ctx.input.current_mouse_state.insert(btn);
}

pub(crate) fn set_mouse_button_up(ctx: &mut Context, btn: MouseButton) {
    ctx.input.current_mouse_state.remove(&btn);
}

pub(crate) fn set_mouse_position(ctx: &mut Context, position: Vec2) {
    ctx.input.mouse_position = position;
}

pub(crate) fn set_text_input(ctx: &mut Context, text: Option<String>) {
    ctx.input.current_text_input = text;
}

pub(crate) fn add_gamepad(ctx: &mut Context, platform_id: i32) -> usize {
    for (i, slot) in ctx.input.pads.iter_mut().enumerate() {
        if slot.is_none() {
            *slot = Some(GamepadState::new(platform_id));
            return i;
        }
    }

    // There wasn't an existing free slot...
    let i = ctx.input.pads.len();
    ctx.input.pads.push(Some(GamepadState::new(platform_id)));
    i
}

pub(crate) fn remove_gamepad(ctx: &mut Context, gamepad_index: usize) {
    ctx.input.pads[gamepad_index] = None;
}

pub(crate) fn get_gamepad(ctx: &Context, gamepad_index: usize) -> Option<&GamepadState> {
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
        Some(pad)
    } else {
        None
    }
}

pub(crate) fn get_gamepad_mut(
    ctx: &mut Context,
    gamepad_index: usize,
) -> Option<&mut GamepadState> {
    if let Some(Some(pad)) = ctx.input.pads.get_mut(gamepad_index) {
        Some(pad)
    } else {
        None
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

/// Returns an iterator of the keys that are currently down.
pub fn get_keys_down(ctx: &Context) -> impl Iterator<Item = &Key> {
    ctx.input.current_key_state.iter()
}

/// Returns an iterator of the keys that were pressed this tick.
pub fn get_keys_pressed(ctx: &Context) -> impl Iterator<Item = &Key> {
    ctx.input
        .current_key_state
        .difference(&ctx.input.previous_key_state)
}

/// Returns an iterator of the keys that were released this tick.
pub fn get_keys_released(ctx: &Context) -> impl Iterator<Item = &Key> {
    ctx.input
        .previous_key_state
        .difference(&ctx.input.current_key_state)
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
    let internal_width = graphics::get_internal_width(ctx) as f32;
    let screen_rect = graphics::get_screen_rect(ctx);

    ((ctx.input.mouse_position.x - screen_rect.x) / screen_rect.width) * internal_width
}

/// Get the Y co-ordinate of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_y(ctx: &Context) -> f32 {
    let internal_height = graphics::get_internal_height(ctx) as f32;
    let screen_rect = graphics::get_screen_rect(ctx);

    ((ctx.input.mouse_position.y - screen_rect.y) / screen_rect.height) * internal_height
}

/// Get the position of the mouse.
///
/// If the screen is scaled, the returned value will be relative to the original size.
pub fn get_mouse_position(ctx: &Context) -> Vec2 {
    Vec2::new(get_mouse_x(ctx), get_mouse_y(ctx))
}

/// Returns true if the specified gamepad is currently connected.
pub fn is_gamepad_connected(ctx: &Context, gamepad_index: usize) -> bool {
    get_gamepad(ctx, gamepad_index).is_some()
}

/// Returns the name of the specified gamepad, or `None` if it is not connected.
pub fn get_gamepad_name(ctx: &Context, gamepad_index: usize) -> Option<String> {
    get_gamepad(ctx, gamepad_index)
        .map(|g| g.platform_id)
        .map(|id| ctx.platform.get_gamepad_name(id))
}

/// Returns true if the specified gamepad button is currently down.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_down(ctx: &Context, gamepad_index: usize, button: GamepadButton) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        pad.current_button_state.contains(&button)
    } else {
        false
    }
}

/// Returns true if the specified gamepad button is currently up.
///
/// If the gamepad is disconnected, this will always return `true`.
pub fn is_gamepad_button_up(ctx: &Context, gamepad_index: usize, button: GamepadButton) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        !pad.current_button_state.contains(&button)
    } else {
        true
    }
}

/// Returns true if the specified gamepad button was pressed this tick.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_pressed(
    ctx: &Context,
    gamepad_index: usize,
    button: GamepadButton,
) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        !pad.previous_button_state.contains(&button) && pad.current_button_state.contains(&button)
    } else {
        false
    }
}

/// Returns true if the specified gamepad button was released this tick.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_released(
    ctx: &Context,
    gamepad_index: usize,
    button: GamepadButton,
) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        pad.previous_button_state.contains(&button) && !pad.current_button_state.contains(&button)
    } else {
        false
    }
}

enum GamepadIterator<T> {
    Disconnected,
    Connected(T),
}

impl<T> Iterator for GamepadIterator<T>
where
    T: Iterator,
{
    type Item = T::Item;

    fn next(&mut self) -> Option<T::Item> {
        match self {
            GamepadIterator::Disconnected => None,
            GamepadIterator::Connected(i) => i.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            GamepadIterator::Disconnected => (0, Some(0)),
            GamepadIterator::Connected(i) => i.size_hint(),
        }
    }
}

/// Returns an iterator of the buttons that are currently down on the specified gamepad.
///
/// If the gamepad is disconnected, an empty iterator will be returned.
pub fn get_gamepad_buttons_down(
    ctx: &Context,
    gamepad_index: usize,
) -> impl Iterator<Item = &GamepadButton> {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        GamepadIterator::Connected(pad.current_button_state.iter())
    } else {
        GamepadIterator::Disconnected
    }
}

/// Returns an iterator of the buttons that were pressed this tick on the specified gamepad.
///
/// If the gamepad is disconnected, an empty iterator will be returned.
pub fn get_gamepad_buttons_pressed(
    ctx: &Context,
    gamepad_index: usize,
) -> impl Iterator<Item = &GamepadButton> {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        GamepadIterator::Connected(
            pad.current_button_state
                .difference(&pad.previous_button_state),
        )
    } else {
        GamepadIterator::Disconnected
    }
}

/// Returns an iterator of the buttons that were released this tick on the specified gamepad.
///
/// If the gamepad is disconnected, an empty iterator will be returned.
pub fn get_gamepad_buttons_released(
    ctx: &Context,
    gamepad_index: usize,
) -> impl Iterator<Item = &GamepadButton> {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        GamepadIterator::Connected(
            pad.previous_button_state
                .difference(&pad.current_button_state),
        )
    } else {
        GamepadIterator::Disconnected
    }
}

/// Returns the current position of the specified gamepad axis.
///
/// If the gamepad is disconnected, this will always return `0.0`.
pub fn get_gamepad_axis_position(ctx: &Context, gamepad_index: usize, axis: GamepadAxis) -> f32 {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        if let Some(value) = pad.current_axis_state.get(&axis) {
            *value
        } else {
            0.0
        }
    } else {
        0.0
    }
}

/// Returns the current position of the specified gamepad control stick.
///
/// If the gamepad is disconnected, this will always return `(0.0, 0.0)`.
pub fn get_gamepad_stick_position(
    ctx: &Context,
    gamepad_index: usize,
    stick: GamepadStick,
) -> Vec2 {
    let (x_axis, y_axis) = match stick {
        GamepadStick::LeftStick => (GamepadAxis::LeftStickX, GamepadAxis::LeftStickY),
        GamepadStick::RightStick => (GamepadAxis::RightStickX, GamepadAxis::RightStickY),
    };

    Vec2::new(
        get_gamepad_axis_position(ctx, gamepad_index, x_axis),
        get_gamepad_axis_position(ctx, gamepad_index, y_axis),
    )
}

/// Returns whether or not the specified gamepad supports vibration.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_vibration_supported(ctx: &Context, gamepad_index: usize) -> bool {
    if let Some(pad) = get_gamepad(ctx, gamepad_index) {
        ctx.platform.is_gamepad_vibration_supported(pad.platform_id)
    } else {
        false
    }
}

/// Sets the specified gamepad's motors to vibrate indefinitely.
pub fn set_gamepad_vibration(ctx: &mut Context, gamepad_index: usize, strength: f32) {
    if let Some(platform_id) = get_gamepad(ctx, gamepad_index).map(|g| g.platform_id) {
        ctx.platform.set_gamepad_vibration(platform_id, strength);
    }
}

/// Sets the specified gamepad's motors to vibrate for a set duration, specified in milliseconds.
/// After this time has passed, the vibration will automatically stop.
pub fn start_gamepad_vibration(
    ctx: &mut Context,
    gamepad_index: usize,
    strength: f32,
    duration: u32,
) {
    if let Some(platform_id) = get_gamepad(ctx, gamepad_index).map(|g| g.platform_id) {
        ctx.platform
            .start_gamepad_vibration(platform_id, strength, duration);
    }
}

/// Stops the specified gamepad's motors from vibrating.
pub fn stop_gamepad_vibration(ctx: &mut Context, gamepad_index: usize) {
    if let Some(platform_id) = get_gamepad(ctx, gamepad_index).map(|g| g.platform_id) {
        ctx.platform.stop_gamepad_vibration(platform_id);
    }
}
