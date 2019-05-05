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
use sdl2::controller::{Axis as SdlAxis, Button as SdlButton, GameController};
use sdl2::event::Event;
use sdl2::haptic::Haptic;
use sdl2::sys::SDL_HAPTIC_INFINITY;
use sdl2::{GameControllerSubsystem, HapticSubsystem, JoystickSubsystem, Sdl};

use crate::error::{Result, TetraError};
use crate::glm::Vec2;
use crate::graphics;
use crate::Context;

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

impl From<SdlButton> for GamepadButton {
    fn from(button: SdlButton) -> GamepadButton {
        match button {
            SdlButton::A => GamepadButton::A,
            SdlButton::B => GamepadButton::B,
            SdlButton::X => GamepadButton::X,
            SdlButton::Y => GamepadButton::Y,
            SdlButton::DPadUp => GamepadButton::Up,
            SdlButton::DPadDown => GamepadButton::Down,
            SdlButton::DPadLeft => GamepadButton::Left,
            SdlButton::DPadRight => GamepadButton::Right,
            SdlButton::LeftShoulder => GamepadButton::LeftShoulder,
            SdlButton::LeftStick => GamepadButton::LeftStick,
            SdlButton::RightShoulder => GamepadButton::RightShoulder,
            SdlButton::RightStick => GamepadButton::RightStick,
            SdlButton::Start => GamepadButton::Start,
            SdlButton::Back => GamepadButton::Back,
            SdlButton::Guide => GamepadButton::Guide,
        }
    }
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

impl From<GamepadAxis> for SdlAxis {
    fn from(axis: GamepadAxis) -> SdlAxis {
        match axis {
            GamepadAxis::LeftStickX => SdlAxis::LeftX,
            GamepadAxis::LeftStickY => SdlAxis::LeftY,
            GamepadAxis::LeftTrigger => SdlAxis::TriggerLeft,
            GamepadAxis::RightStickX => SdlAxis::RightX,
            GamepadAxis::RightStickY => SdlAxis::RightY,
            GamepadAxis::RightTrigger => SdlAxis::TriggerRight,
        }
    }
}

impl From<SdlAxis> for GamepadAxis {
    fn from(axis: SdlAxis) -> GamepadAxis {
        match axis {
            SdlAxis::LeftX => GamepadAxis::LeftStickX,
            SdlAxis::LeftY => GamepadAxis::LeftStickY,
            SdlAxis::TriggerLeft => GamepadAxis::LeftTrigger,
            SdlAxis::RightX => GamepadAxis::RightStickX,
            SdlAxis::RightY => GamepadAxis::RightStickY,
            SdlAxis::TriggerRight => GamepadAxis::RightTrigger,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum GamepadStick {
    LeftStick,
    RightStick,
}

struct GamepadState {
    // NOTE: The SDL docs say to close the haptic device before the joystick, so
    // I've ordered the fields accordingly.
    sdl_haptic: Option<Haptic>,
    sdl_controller: GameController,

    current_button_state: HashSet<GamepadButton>,
    previous_button_state: HashSet<GamepadButton>,
    current_axis_state: HashMap<GamepadAxis, f32>,
}

impl GamepadState {
    pub(crate) fn new(sdl_controller: GameController, sdl_haptic: Option<Haptic>) -> GamepadState {
        GamepadState {
            sdl_haptic,
            sdl_controller,

            current_button_state: HashSet::new(),
            previous_button_state: HashSet::new(),
            current_axis_state: HashMap::new(),
        }
    }
}

pub(crate) struct InputContext {
    current_key_state: HashSet<Key>,
    previous_key_state: HashSet<Key>,
    current_text_input: Option<String>,

    current_mouse_state: HashSet<MouseButton>,
    previous_mouse_state: HashSet<MouseButton>,
    mouse_position: Vec2,

    controller_sys: GameControllerSubsystem,
    _joystick_sys: JoystickSubsystem,
    haptic_sys: HapticSubsystem,
    pads: Vec<Option<GamepadState>>,
    sdl_pad_indexes: HashMap<i32, usize>,
}

impl InputContext {
    pub(crate) fn new(sdl: &Sdl) -> Result<InputContext> {
        let _joystick_sys = sdl.joystick().map_err(TetraError::Sdl)?;
        let controller_sys = sdl.game_controller().map_err(TetraError::Sdl)?;
        let haptic_sys = sdl.haptic().map_err(TetraError::Sdl)?;

        sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");

        Ok(InputContext {
            current_key_state: HashSet::new(),
            previous_key_state: HashSet::new(),
            current_text_input: None,

            current_mouse_state: HashSet::new(),
            previous_mouse_state: HashSet::new(),
            mouse_position: Vec2::zeros(),

            controller_sys,
            _joystick_sys,
            haptic_sys,
            pads: Vec::new(),
            sdl_pad_indexes: HashMap::new(),
        })
    }
}

pub(crate) fn handle_event(ctx: &mut Context, event: Event) -> Result {
    match event {
        Event::KeyDown {
            keycode: Some(k), ..
        } => {
            if let Key::Escape = k {
                if ctx.quit_on_escape {
                    ctx.running = false;
                }
            }

            ctx.input.current_key_state.insert(k);
        }
        Event::KeyUp {
            keycode: Some(k), ..
        } => {
            // TODO: This can cause some inputs to be missed at low tick rates.
            // Could consider buffering input releases like Otter2D does?
            ctx.input.current_key_state.remove(&k);
        }
        Event::MouseButtonDown { mouse_btn, .. } => {
            ctx.input.current_mouse_state.insert(mouse_btn);
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            ctx.input.current_mouse_state.remove(&mouse_btn);
        }
        Event::MouseMotion { x, y, .. } => ctx.input.mouse_position = Vec2::new(x as f32, y as f32),
        Event::TextInput { text, .. } => {
            ctx.input.current_text_input = Some(text);
        }
        Event::ControllerDeviceAdded { which, .. } => {
            let controller = ctx.input.controller_sys.open(which)?;
            let haptic = ctx.input.haptic_sys.open_from_joystick_id(which).ok();

            let id = controller.instance_id();

            for (i, slot) in ctx.input.pads.iter_mut().enumerate() {
                if slot.is_none() {
                    ctx.input.sdl_pad_indexes.insert(id, i);
                    *slot = Some(GamepadState::new(controller, haptic));
                    return Ok(());
                }
            }

            // There wasn't an existing free slot...
            ctx.input.sdl_pad_indexes.insert(id, ctx.input.pads.len());
            ctx.input
                .pads
                .push(Some(GamepadState::new(controller, haptic)));
        }
        Event::ControllerDeviceRemoved { which, .. } => {
            let i = ctx.input.sdl_pad_indexes.remove(&which).unwrap();
            ctx.input.pads[i] = None;
        }
        Event::ControllerButtonDown { which, button, .. } => {
            if let Some(i) = ctx.input.sdl_pad_indexes.get(&which) {
                if let Some(Some(pad)) = ctx.input.pads.get_mut(*i) {
                    pad.current_button_state.insert(button.into());
                }
            }
        }
        Event::ControllerButtonUp { which, button, .. } => {
            if let Some(i) = ctx.input.sdl_pad_indexes.get(&which) {
                if let Some(Some(pad)) = ctx.input.pads.get_mut(*i) {
                    pad.current_button_state.remove(&button.into());
                }
            }
        }
        Event::ControllerAxisMotion {
            which, axis, value, ..
        } => {
            if let Some(i) = ctx.input.sdl_pad_indexes.get(&which) {
                if let Some(Some(pad)) = ctx.input.pads.get_mut(*i) {
                    pad.current_axis_state
                        .insert(axis.into(), f32::from(value) / 32767.0);

                    match axis {
                        SdlAxis::TriggerLeft => {
                            if value > 0 {
                                pad.current_button_state.insert(GamepadButton::LeftTrigger);
                            } else {
                                pad.current_button_state.remove(&GamepadButton::LeftTrigger);
                            }
                        }
                        SdlAxis::TriggerRight => {
                            if value > 0 {
                                pad.current_button_state.insert(GamepadButton::RightTrigger);
                            } else {
                                pad.current_button_state
                                    .remove(&GamepadButton::RightTrigger);
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        _ => {}
    }

    Ok(())
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
    if let Some(Some(_)) = ctx.input.pads.get(gamepad_index) {
        true
    } else {
        false
    }
}

/// Returns the name of the specified gamepad, or `None` if it is not connected.
pub fn get_gamepad_name(ctx: &Context, gamepad_index: usize) -> Option<String> {
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
        Some(pad.sdl_controller.name())
    } else {
        None
    }
}

/// Returns true if the specified gamepad button is currently down.
///
/// If the gamepad is disconnected, this will always return `false`.
pub fn is_gamepad_button_down(ctx: &Context, gamepad_index: usize, button: GamepadButton) -> bool {
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
        pad.current_button_state.contains(&button)
    } else {
        false
    }
}

/// Returns true if the specified gamepad button is currently up.
///
/// If the gamepad is disconnected, this will always return `true`.
pub fn is_gamepad_button_up(ctx: &Context, gamepad_index: usize, button: GamepadButton) -> bool {
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
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
    if let Some(Some(pad)) = ctx.input.pads.get(gamepad_index) {
        pad.sdl_haptic.is_some()
    } else {
        false
    }
}

/// Sets the specified gamepad's motors to vibrate indefinitely.
pub fn set_gamepad_vibration(ctx: &mut Context, gamepad_index: usize, strength: f32) {
    start_gamepad_vibration(ctx, gamepad_index, strength, SDL_HAPTIC_INFINITY);
}

/// Sets the specified gamepad's motors to vibrate for a set duration, specified in milliseconds.
/// After this time has passed, the vibration will automatically stop.
pub fn start_gamepad_vibration(
    ctx: &mut Context,
    gamepad_index: usize,
    strength: f32,
    duration: u32,
) {
    if let Some(Some(pad)) = ctx.input.pads.get_mut(gamepad_index) {
        if let Some(haptic) = &mut pad.sdl_haptic {
            haptic.rumble_play(strength, duration);
        }
    }
}

/// Stops the specified gamepad's motors from vibrating.
pub fn stop_gamepad_vibration(ctx: &mut Context, gamepad_index: usize) {
    start_gamepad_vibration(ctx, gamepad_index, 0.0, SDL_HAPTIC_INFINITY);
}
