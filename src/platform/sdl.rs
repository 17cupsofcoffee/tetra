use glow::native::Context as GlContext;
use hashbrown::HashMap;
use sdl2::controller::{Axis as SdlGamepadAxis, Button as SdlGamepadButton, GameController};
use sdl2::event::{Event, WindowEvent};
use sdl2::haptic::Haptic;
use sdl2::keyboard::Keycode as SdlKey;
use sdl2::mouse::MouseButton as SdlMouseButton;
use sdl2::sys::SDL_HAPTIC_INFINITY;
use sdl2::video::{FullscreenType, GLContext as SdlGlContext, GLProfile, Window, WindowBuildError};
use sdl2::{
    GameControllerSubsystem, HapticSubsystem, IntegerOrSdlError, JoystickSubsystem, Sdl,
    VideoSubsystem,
};

use crate::error::{Result, TetraError};
use crate::graphics::{self, Vec2};
use crate::input::{self, GamepadAxis, GamepadButton, Key, MouseButton};
use crate::window;
use crate::{Context, ContextBuilder, State};

pub struct Platform {
    sdl: Sdl,
    window: Window,

    _video_sys: VideoSubsystem,
    controller_sys: GameControllerSubsystem,
    _joystick_sys: JoystickSubsystem,
    haptic_sys: HapticSubsystem,
    _gl_sys: SdlGlContext,

    controllers: HashMap<i32, SdlController>,

    window_width: i32,
    window_height: i32,
    fullscreen: bool,
}

struct SdlController {
    // NOTE: The SDL docs say to close the haptic device before the joystick, so
    // I've ordered the fields accordingly.
    haptic: Option<Haptic>,
    controller: GameController,
    slot: usize,
}

impl Platform {
    pub fn new(builder: &ContextBuilder) -> Result<(Platform, GlContext, i32, i32)> {
        let sdl = sdl2::init().map_err(TetraError::Sdl)?;

        let video_sys = sdl.video().map_err(TetraError::Sdl)?;
        let joystick_sys = sdl.joystick().map_err(TetraError::Sdl)?;
        let controller_sys = sdl.game_controller().map_err(TetraError::Sdl)?;
        let haptic_sys = sdl.haptic().map_err(TetraError::Sdl)?;

        sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");

        let gl_attr = video_sys.gl_attr();

        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);
        gl_attr.set_red_size(8);
        gl_attr.set_green_size(8);
        gl_attr.set_blue_size(8);
        gl_attr.set_alpha_size(8);
        gl_attr.set_double_buffer(true);
        // TODO: Will need to add some more here if we start using the depth/stencil buffers

        let (mut window_width, mut window_height) = if let Some(size) = builder.window_size {
            size
        } else if let Some(scale) = builder.window_scale {
            (
                builder.internal_width * scale,
                builder.internal_height * scale,
            )
        } else {
            (builder.internal_width, builder.internal_height)
        };

        let mut window_builder =
            video_sys.window(&builder.title, window_width as u32, window_height as u32);

        window_builder.hidden().position_centered().opengl();

        if builder.resizable {
            window_builder.resizable();
        }

        if builder.borderless {
            window_builder.borderless();
        }

        sdl.mouse().show_cursor(builder.show_mouse);

        let mut window = window_builder.build()?;

        // We wait until the window has been created to fiddle with this stuff as:
        // a) we don't want to blow away the window size settings
        // b) we don't know what monitor they're on until the window is created

        if builder.maximized {
            window.maximize();
            let size = window.drawable_size();
            window_width = size.0 as i32;
            window_height = size.1 as i32;
        } else if builder.minimized {
            window.minimize();
            let size = window.drawable_size();
            window_width = size.0 as i32;
            window_height = size.1 as i32;
        }

        if builder.fullscreen {
            window
                .display_mode()
                .and_then(|m| {
                    window_width = m.w;
                    window_height = m.h;
                    window.set_fullscreen(FullscreenType::Desktop)
                })
                .map_err(TetraError::Sdl)?;
        }

        let gl_sys = window.gl_create_context().map_err(TetraError::OpenGl)?;
        let gl_ctx =
            GlContext::from_loader_function(|s| video_sys.gl_get_proc_address(s) as *const _);

        video_sys
            .gl_set_swap_interval(if builder.vsync { 1 } else { 0 })
            .map_err(TetraError::Sdl)?;

        let platform = Platform {
            sdl,
            window,

            _video_sys: video_sys,
            controller_sys,
            _joystick_sys: joystick_sys,
            haptic_sys,
            _gl_sys: gl_sys,

            controllers: HashMap::new(),

            window_width,
            window_height,
            fullscreen: builder.fullscreen,
        };

        Ok((platform, gl_ctx, window_width, window_height))
    }
}

pub fn run_loop<S>(mut ctx: Context, mut state: S, frame: fn(&mut Context, &mut S))
where
    S: State,
{
    ctx.running = true;
    ctx.platform.window.show();

    while ctx.running {
        frame(&mut ctx, &mut state);
    }

    ctx.platform.window.hide();
}

pub fn handle_events(ctx: &mut Context) -> Result {
    let mut events = ctx.platform.sdl.event_pump().map_err(TetraError::Sdl)?;

    for event in events.poll_iter() {
        match event {
            Event::Quit { .. } => ctx.running = false, // TODO: Add a way to override this

            Event::Window { win_event, .. } => {
                if let WindowEvent::SizeChanged(width, height) = win_event {
                    ctx.platform.window_width = width;
                    ctx.platform.window_height = height;
                    graphics::set_window_projection(ctx, width, height);
                }
            }

            Event::KeyDown {
                keycode: Some(k), ..
            } => {
                if let SdlKey::Escape = k {
                    if ctx.quit_on_escape {
                        ctx.running = false;
                    }
                }

                if let Some(k) = into_key(k) {
                    input::set_key_down(ctx, k);
                }
            }

            Event::KeyUp {
                keycode: Some(k), ..
            } => {
                if let Some(k) = into_key(k) {
                    // TODO: This can cause some inputs to be missed at low tick rates.
                    // Could consider buffering input releases like Otter2D does?
                    input::set_key_up(ctx, k);
                }
            }

            Event::MouseButtonDown { mouse_btn, .. } => {
                if let Some(b) = into_mouse_button(mouse_btn) {
                    input::set_mouse_button_down(ctx, b);
                }
            }

            Event::MouseButtonUp { mouse_btn, .. } => {
                if let Some(b) = into_mouse_button(mouse_btn) {
                    input::set_mouse_button_up(ctx, b);
                }
            }

            Event::MouseMotion { x, y, .. } => {
                input::set_mouse_position(ctx, Vec2::new(x as f32, y as f32));
            }

            Event::TextInput { text, .. } => {
                input::set_text_input(ctx, Some(text));
            }

            Event::ControllerDeviceAdded { which, .. } => {
                let controller = ctx.platform.controller_sys.open(which)?;
                let haptic = ctx.platform.haptic_sys.open_from_joystick_id(which).ok();

                let id = controller.instance_id();
                let slot = input::add_gamepad(ctx, id);

                ctx.platform.controllers.insert(
                    id,
                    SdlController {
                        controller,
                        haptic,
                        slot,
                    },
                );
            }

            Event::ControllerDeviceRemoved { which, .. } => {
                let controller = ctx.platform.controllers.remove(&which).unwrap();
                input::remove_gamepad(ctx, controller.slot);
            }

            Event::ControllerButtonDown { which, button, .. } => {
                if let Some(slot) = ctx.platform.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        pad.set_button_down(button.into());
                    }
                }
            }

            Event::ControllerButtonUp { which, button, .. } => {
                if let Some(slot) = ctx.platform.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        // TODO: This can cause some inputs to be missed at low tick rates.
                        // Could consider buffering input releases like Otter2D does?
                        pad.set_button_up(button.into());
                    }
                }
            }

            Event::ControllerAxisMotion {
                which, axis, value, ..
            } => {
                if let Some(slot) = ctx.platform.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        pad.set_axis_position(axis.into(), f32::from(value) / 32767.0);

                        match axis {
                            SdlGamepadAxis::TriggerLeft => {
                                if value > 0 {
                                    pad.set_button_down(GamepadButton::LeftTrigger);
                                } else {
                                    pad.set_button_up(GamepadButton::LeftTrigger);
                                }
                            }

                            SdlGamepadAxis::TriggerRight => {
                                if value > 0 {
                                    pad.set_button_down(GamepadButton::RightTrigger);
                                } else {
                                    pad.set_button_up(GamepadButton::RightTrigger);
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }

            _ => {}
        }
    }

    Ok(())
}

pub fn get_window_title(ctx: &Context) -> &str {
    ctx.platform.window.title()
}

pub fn set_window_title<S>(ctx: &mut Context, title: S)
where
    S: AsRef<str>,
{
    ctx.platform.window.set_title(title.as_ref()).unwrap();
}

pub fn get_window_width(ctx: &Context) -> i32 {
    ctx.platform.window_width
}

pub fn get_window_height(ctx: &Context) -> i32 {
    ctx.platform.window_height
}

pub fn get_window_size(ctx: &Context) -> (i32, i32) {
    (ctx.platform.window_width, ctx.platform.window_height)
}

pub fn set_window_size(ctx: &mut Context, width: i32, height: i32) {
    ctx.platform.window_width = width;
    ctx.platform.window_height = height;

    ctx.platform
        .window
        .set_size(width as u32, height as u32)
        .unwrap();
}

pub fn toggle_fullscreen(ctx: &mut Context) -> Result {
    if ctx.platform.fullscreen {
        disable_fullscreen(ctx)
    } else {
        enable_fullscreen(ctx)
    }
}

pub fn enable_fullscreen(ctx: &mut Context) -> Result {
    if !ctx.platform.fullscreen {
        ctx.platform
            .window
            .display_mode()
            .and_then(|m| {
                window::set_size(ctx, m.w, m.h);
                ctx.platform.window.set_fullscreen(FullscreenType::Desktop)
            })
            .map(|_| ())
            .map_err(TetraError::Sdl)
    } else {
        Ok(())
    }
}

pub fn disable_fullscreen(ctx: &mut Context) -> Result {
    if ctx.platform.fullscreen {
        ctx.platform
            .window
            .set_fullscreen(FullscreenType::Off)
            .map(|_| {
                let size = ctx.platform.window.drawable_size();
                window::set_size(ctx, size.0 as i32, size.1 as i32);
            })
            .map_err(TetraError::Sdl)
    } else {
        Ok(())
    }
}

pub fn is_fullscreen(ctx: &Context) -> bool {
    ctx.platform.fullscreen
}

pub fn set_mouse_visible(ctx: &mut Context, mouse_visible: bool) {
    ctx.platform.sdl.mouse().show_cursor(mouse_visible);
}

pub fn is_mouse_visible(ctx: &Context) -> bool {
    ctx.platform.sdl.mouse().is_cursor_showing()
}

pub fn swap_buffers(ctx: &Context) {
    ctx.platform.window.gl_swap_window();
}

pub fn get_gamepad_name(ctx: &Context, platform_id: i32) -> String {
    ctx.platform.controllers[&platform_id].controller.name()
}

pub fn is_gamepad_vibration_supported(ctx: &Context, platform_id: i32) -> bool {
    ctx.platform.controllers[&platform_id].haptic.is_some()
}

pub fn set_gamepad_vibration(ctx: &mut Context, platform_id: i32, strength: f32) {
    start_gamepad_vibration(ctx, platform_id, strength, SDL_HAPTIC_INFINITY);
}

pub fn start_gamepad_vibration(ctx: &mut Context, platform_id: i32, strength: f32, duration: u32) {
    if let Some(haptic) = ctx
        .platform
        .controllers
        .get_mut(&platform_id)
        .and_then(|c| c.haptic.as_mut())
    {
        haptic.rumble_play(strength, duration);
    }
}

pub fn stop_gamepad_vibration(ctx: &mut Context, platform_id: i32) {
    if let Some(haptic) = ctx
        .platform
        .controllers
        .get_mut(&platform_id)
        .and_then(|c| c.haptic.as_mut())
    {
        haptic.rumble_stop();
    }
}

// TODO: Replace these with TryFrom once we're on a high enough minimum Rust version?

fn into_mouse_button(button: SdlMouseButton) -> Option<MouseButton> {
    match button {
        SdlMouseButton::Left => Some(MouseButton::Left),
        SdlMouseButton::Middle => Some(MouseButton::Middle),
        SdlMouseButton::Right => Some(MouseButton::Right),
        SdlMouseButton::X1 => Some(MouseButton::X1),
        SdlMouseButton::X2 => Some(MouseButton::X2),
        _ => None,
    }
}

fn into_key(key: SdlKey) -> Option<Key> {
    match key {
        SdlKey::A => Some(Key::A),
        SdlKey::B => Some(Key::B),
        SdlKey::C => Some(Key::C),
        SdlKey::D => Some(Key::D),
        SdlKey::E => Some(Key::E),
        SdlKey::F => Some(Key::F),
        SdlKey::G => Some(Key::G),
        SdlKey::H => Some(Key::H),
        SdlKey::I => Some(Key::I),
        SdlKey::J => Some(Key::J),
        SdlKey::K => Some(Key::K),
        SdlKey::L => Some(Key::L),
        SdlKey::M => Some(Key::M),
        SdlKey::N => Some(Key::N),
        SdlKey::O => Some(Key::O),
        SdlKey::P => Some(Key::P),
        SdlKey::Q => Some(Key::Q),
        SdlKey::R => Some(Key::R),
        SdlKey::S => Some(Key::S),
        SdlKey::T => Some(Key::T),
        SdlKey::U => Some(Key::U),
        SdlKey::V => Some(Key::V),
        SdlKey::W => Some(Key::W),
        SdlKey::X => Some(Key::X),
        SdlKey::Y => Some(Key::Y),
        SdlKey::Z => Some(Key::Z),

        SdlKey::Num0 => Some(Key::Num0),
        SdlKey::Num1 => Some(Key::Num1),
        SdlKey::Num2 => Some(Key::Num2),
        SdlKey::Num3 => Some(Key::Num3),
        SdlKey::Num4 => Some(Key::Num4),
        SdlKey::Num5 => Some(Key::Num5),
        SdlKey::Num6 => Some(Key::Num6),
        SdlKey::Num7 => Some(Key::Num7),
        SdlKey::Num8 => Some(Key::Num8),
        SdlKey::Num9 => Some(Key::Num9),

        SdlKey::F1 => Some(Key::F1),
        SdlKey::F2 => Some(Key::F2),
        SdlKey::F3 => Some(Key::F3),
        SdlKey::F4 => Some(Key::F4),
        SdlKey::F5 => Some(Key::F5),
        SdlKey::F6 => Some(Key::F6),
        SdlKey::F7 => Some(Key::F7),
        SdlKey::F8 => Some(Key::F8),
        SdlKey::F9 => Some(Key::F9),
        SdlKey::F10 => Some(Key::F10),
        SdlKey::F11 => Some(Key::F11),
        SdlKey::F12 => Some(Key::F12),
        SdlKey::F13 => Some(Key::F13),
        SdlKey::F14 => Some(Key::F14),
        SdlKey::F15 => Some(Key::F15),
        SdlKey::F16 => Some(Key::F16),
        SdlKey::F17 => Some(Key::F17),
        SdlKey::F18 => Some(Key::F18),
        SdlKey::F19 => Some(Key::F19),
        SdlKey::F20 => Some(Key::F20),
        SdlKey::F21 => Some(Key::F21),
        SdlKey::F22 => Some(Key::F22),
        SdlKey::F23 => Some(Key::F23),
        SdlKey::F24 => Some(Key::F24),

        SdlKey::NumLockClear => Some(Key::NumLock),
        SdlKey::Kp1 => Some(Key::NumPad1),
        SdlKey::Kp2 => Some(Key::NumPad2),
        SdlKey::Kp3 => Some(Key::NumPad3),
        SdlKey::Kp4 => Some(Key::NumPad4),
        SdlKey::Kp5 => Some(Key::NumPad5),
        SdlKey::Kp6 => Some(Key::NumPad6),
        SdlKey::Kp7 => Some(Key::NumPad7),
        SdlKey::Kp8 => Some(Key::NumPad8),
        SdlKey::Kp9 => Some(Key::NumPad9),
        SdlKey::Kp0 => Some(Key::NumPad0),
        SdlKey::KpPlus => Some(Key::NumPadPlus),
        SdlKey::KpMinus => Some(Key::NumPadMinus),
        SdlKey::KpMultiply => Some(Key::NumPadMultiply),
        SdlKey::KpDivide => Some(Key::NumPadDivide),
        SdlKey::KpEnter => Some(Key::NumPadEnter),

        SdlKey::LCtrl => Some(Key::LeftCtrl),
        SdlKey::LShift => Some(Key::LeftShift),
        SdlKey::LAlt => Some(Key::LeftAlt),
        SdlKey::RCtrl => Some(Key::RightCtrl),
        SdlKey::RShift => Some(Key::RightShift),
        SdlKey::RAlt => Some(Key::RightAlt),

        SdlKey::Up => Some(Key::Up),
        SdlKey::Down => Some(Key::Down),
        SdlKey::Left => Some(Key::Left),
        SdlKey::Right => Some(Key::Right),

        SdlKey::Ampersand => Some(Key::Ampersand),
        SdlKey::Asterisk => Some(Key::Asterisk),
        SdlKey::At => Some(Key::At),
        SdlKey::Backquote => Some(Key::Backquote),
        SdlKey::Backslash => Some(Key::Backslash),
        SdlKey::Backspace => Some(Key::Backspace),
        SdlKey::CapsLock => Some(Key::CapsLock),
        SdlKey::Caret => Some(Key::Caret),
        SdlKey::Colon => Some(Key::Colon),
        SdlKey::Comma => Some(Key::Comma),
        SdlKey::Delete => Some(Key::Delete),
        SdlKey::Dollar => Some(Key::Dollar),
        SdlKey::Quotedbl => Some(Key::DoubleQuote),
        SdlKey::End => Some(Key::End),
        SdlKey::Return => Some(Key::Enter),
        SdlKey::Equals => Some(Key::Equals),
        SdlKey::Escape => Some(Key::Escape),
        SdlKey::Exclaim => Some(Key::Exclaim),
        SdlKey::Greater => Some(Key::GreaterThan),
        SdlKey::Hash => Some(Key::Hash),
        SdlKey::Home => Some(Key::Home),
        SdlKey::Insert => Some(Key::Insert),
        SdlKey::LeftBracket => Some(Key::LeftBracket),
        SdlKey::LeftParen => Some(Key::LeftParen),
        SdlKey::Less => Some(Key::LessThan),
        SdlKey::Minus => Some(Key::Minus),
        SdlKey::PageDown => Some(Key::PageDown),
        SdlKey::PageUp => Some(Key::PageUp),
        SdlKey::Pause => Some(Key::Pause),
        SdlKey::Percent => Some(Key::Percent),
        SdlKey::Period => Some(Key::Period),
        SdlKey::Plus => Some(Key::Plus),
        SdlKey::PrintScreen => Some(Key::PrintScreen),
        SdlKey::Question => Some(Key::Question),
        SdlKey::Quote => Some(Key::Quote),
        SdlKey::RightBracket => Some(Key::RightBracket),
        SdlKey::RightParen => Some(Key::RightParen),
        SdlKey::ScrollLock => Some(Key::ScrollLock),
        SdlKey::Semicolon => Some(Key::Semicolon),
        SdlKey::Slash => Some(Key::Slash),
        SdlKey::Space => Some(Key::Space),
        SdlKey::Tab => Some(Key::Tab),
        SdlKey::Underscore => Some(Key::Underscore),

        _ => None,
    }
}

#[doc(hidden)]
impl From<SdlGamepadButton> for GamepadButton {
    fn from(button: SdlGamepadButton) -> GamepadButton {
        match button {
            SdlGamepadButton::A => GamepadButton::A,
            SdlGamepadButton::B => GamepadButton::B,
            SdlGamepadButton::X => GamepadButton::X,
            SdlGamepadButton::Y => GamepadButton::Y,
            SdlGamepadButton::DPadUp => GamepadButton::Up,
            SdlGamepadButton::DPadDown => GamepadButton::Down,
            SdlGamepadButton::DPadLeft => GamepadButton::Left,
            SdlGamepadButton::DPadRight => GamepadButton::Right,
            SdlGamepadButton::LeftShoulder => GamepadButton::LeftShoulder,
            SdlGamepadButton::LeftStick => GamepadButton::LeftStick,
            SdlGamepadButton::RightShoulder => GamepadButton::RightShoulder,
            SdlGamepadButton::RightStick => GamepadButton::RightStick,
            SdlGamepadButton::Start => GamepadButton::Start,
            SdlGamepadButton::Back => GamepadButton::Back,
            SdlGamepadButton::Guide => GamepadButton::Guide,
        }
    }
}

#[doc(hidden)]
impl From<GamepadAxis> for SdlGamepadAxis {
    fn from(axis: GamepadAxis) -> SdlGamepadAxis {
        match axis {
            GamepadAxis::LeftStickX => SdlGamepadAxis::LeftX,
            GamepadAxis::LeftStickY => SdlGamepadAxis::LeftY,
            GamepadAxis::LeftTrigger => SdlGamepadAxis::TriggerLeft,
            GamepadAxis::RightStickX => SdlGamepadAxis::RightX,
            GamepadAxis::RightStickY => SdlGamepadAxis::RightY,
            GamepadAxis::RightTrigger => SdlGamepadAxis::TriggerRight,
        }
    }
}

#[doc(hidden)]
impl From<SdlGamepadAxis> for GamepadAxis {
    fn from(axis: SdlGamepadAxis) -> GamepadAxis {
        match axis {
            SdlGamepadAxis::LeftX => GamepadAxis::LeftStickX,
            SdlGamepadAxis::LeftY => GamepadAxis::LeftStickY,
            SdlGamepadAxis::TriggerLeft => GamepadAxis::LeftTrigger,
            SdlGamepadAxis::RightX => GamepadAxis::RightStickX,
            SdlGamepadAxis::RightY => GamepadAxis::RightStickY,
            SdlGamepadAxis::TriggerRight => GamepadAxis::RightTrigger,
        }
    }
}

#[doc(hidden)]
impl From<WindowBuildError> for TetraError {
    fn from(e: WindowBuildError) -> TetraError {
        TetraError::Sdl(e.to_string())
    }
}

#[doc(hidden)]
impl From<IntegerOrSdlError> for TetraError {
    fn from(e: IntegerOrSdlError) -> TetraError {
        TetraError::Sdl(e.to_string())
    }
}
