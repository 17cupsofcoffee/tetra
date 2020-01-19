// TODO: This file is getting way too huge.
use hashbrown::HashMap;

use glow::Context as GlowContext;
use sdl2::controller::{Axis as SdlGamepadAxis, Button as SdlGamepadButton, GameController};
use sdl2::event::{Event as SdlEvent, WindowEvent};
use sdl2::haptic::Haptic;
use sdl2::keyboard::Keycode as SdlKey;
use sdl2::mouse::MouseButton as SdlMouseButton;
use sdl2::sys::SDL_HAPTIC_INFINITY;
use sdl2::video::{
    FullscreenType, GLContext as SdlGlContext, GLProfile, SwapInterval, Window as SdlWindow,
};
use sdl2::{
    EventPump, GameControllerSubsystem, HapticSubsystem, JoystickSubsystem, Sdl, VideoSubsystem,
};

use crate::error::{Result, TetraError};
use crate::graphics;
use crate::input::{self, GamepadAxis, GamepadButton, GamepadStick, Key, MouseButton};
use crate::math::Vec2;
use crate::{Context, ContextBuilder, Event, State};

struct SdlController {
    // NOTE: The SDL docs say to close the haptic device before the joystick, so
    // I've ordered the fields accordingly.
    haptic: Option<Haptic>,
    controller: GameController,
}

pub struct Window {
    sdl: Sdl,
    sdl_window: SdlWindow,

    event_pump: EventPump,
    video_sys: VideoSubsystem,
    controller_sys: GameControllerSubsystem,
    _joystick_sys: JoystickSubsystem,
    haptic_sys: HapticSubsystem,
    _gl_sys: SdlGlContext,

    controllers: HashMap<i32, SdlController>,

    window_width: i32,
    window_height: i32,
}

impl Window {
    pub fn new(settings: &ContextBuilder) -> Result<(Window, GlowContext, i32, i32)> {
        let sdl = sdl2::init().map_err(TetraError::PlatformError)?;
        let event_pump = sdl.event_pump().map_err(TetraError::PlatformError)?;
        let video_sys = sdl.video().map_err(TetraError::PlatformError)?;
        let joystick_sys = sdl.joystick().map_err(TetraError::PlatformError)?;
        let controller_sys = sdl.game_controller().map_err(TetraError::PlatformError)?;
        let haptic_sys = sdl.haptic().map_err(TetraError::PlatformError)?;

        sdl2::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");

        let gl_attr = video_sys.gl_attr();

        // TODO: Will need to add some more here if we start using the depth/stencil buffers
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);
        gl_attr.set_red_size(8);
        gl_attr.set_green_size(8);
        gl_attr.set_blue_size(8);
        gl_attr.set_alpha_size(8);
        gl_attr.set_double_buffer(true);

        let mut window_builder = video_sys.window(
            &settings.title,
            settings.window_width as u32,
            settings.window_height as u32,
        );

        // The window starts hidden, so that it doesn't look weird if we
        // maximize/minimize/fullscreen the window after it opens.
        window_builder.hidden().position_centered().opengl();

        if settings.resizable {
            window_builder.resizable();
        }

        if settings.borderless {
            window_builder.borderless();
        }

        sdl.mouse().show_cursor(settings.show_mouse);

        let mut sdl_window = window_builder
            .build()
            .map_err(|e| TetraError::PlatformError(e.to_string()))?;

        // We wait until the window has been created to fiddle with this stuff as:
        // a) we don't want to blow away the window size settings
        // b) we don't know what monitor they're on until the window is created

        let mut window_width = settings.window_width;
        let mut window_height = settings.window_height;

        if settings.maximized {
            sdl_window.maximize();
            let size = sdl_window.drawable_size();
            window_width = size.0 as i32;
            window_height = size.1 as i32;
        } else if settings.minimized {
            sdl_window.minimize();
            let size = sdl_window.drawable_size();
            window_width = size.0 as i32;
            window_height = size.1 as i32;
        }

        if settings.fullscreen {
            sdl_window
                .display_mode()
                .and_then(|m| {
                    window_width = m.w;
                    window_height = m.h;
                    sdl_window.set_fullscreen(FullscreenType::Desktop)
                })
                .map_err(TetraError::FailedToChangeDisplayMode)?;
        }

        let gl_sys = sdl_window
            .gl_create_context()
            .map_err(TetraError::PlatformError)?;

        let gl_ctx =
            GlowContext::from_loader_function(|s| video_sys.gl_get_proc_address(s) as *const _);

        video_sys
            .gl_set_swap_interval(if settings.vsync {
                SwapInterval::VSync
            } else {
                SwapInterval::Immediate
            })
            .map_err(TetraError::FailedToChangeDisplayMode)?;

        // Now the window is ready to show to the user.
        sdl_window.show();

        let window = Window {
            sdl,
            sdl_window,

            event_pump,
            video_sys,
            controller_sys,
            _joystick_sys: joystick_sys,
            haptic_sys,
            _gl_sys: gl_sys,

            controllers: HashMap::new(),

            window_width,
            window_height,
        };

        Ok((window, gl_ctx, window_width, window_height))
    }

    pub fn get_window_title(&self) -> &str {
        self.sdl_window.title()
    }

    pub fn set_window_title<S>(&mut self, title: S)
    where
        S: AsRef<str>,
    {
        self.sdl_window.set_title(title.as_ref()).unwrap();
    }

    pub fn get_window_width(&self) -> i32 {
        self.window_width
    }

    pub fn get_window_height(&self) -> i32 {
        self.window_height
    }

    pub fn get_window_size(&self) -> (i32, i32) {
        (self.window_width, self.window_height)
    }

    pub fn set_window_size(&mut self, width: i32, height: i32) -> Result {
        self.window_width = width;
        self.window_height = height;

        self.sdl_window
            .set_size(width as u32, height as u32)
            .map_err(|e| TetraError::FailedToChangeDisplayMode(e.to_string()))
    }

    pub fn set_vsync(&mut self, vsync: bool) -> Result {
        self.video_sys
            .gl_set_swap_interval(if vsync {
                SwapInterval::VSync
            } else {
                SwapInterval::Immediate
            })
            .map_err(TetraError::FailedToChangeDisplayMode)
    }

    pub fn is_vsync_enabled(&self) -> bool {
        self.video_sys.gl_get_swap_interval() != SwapInterval::Immediate
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result {
        if fullscreen {
            self.sdl_window
                .display_mode()
                .map_err(TetraError::FailedToChangeDisplayMode)
                .and_then(|m| self.set_window_size(m.w, m.h))
                .and_then(|_| {
                    self.sdl_window
                        .set_fullscreen(FullscreenType::Desktop)
                        .map_err(TetraError::FailedToChangeDisplayMode)
                })
                .map(|_| ())
        } else {
            self.sdl_window
                .set_fullscreen(FullscreenType::Off)
                .map_err(TetraError::FailedToChangeDisplayMode)
                .and_then(|_| {
                    let size = self.sdl_window.drawable_size();
                    self.set_window_size(size.0 as i32, size.1 as i32)
                })
        }
    }

    pub fn is_fullscreen(&self) -> bool {
        self.sdl_window.fullscreen_state() != FullscreenType::Off
    }

    pub fn set_mouse_visible(&mut self, mouse_visible: bool) -> Result {
        self.sdl.mouse().show_cursor(mouse_visible);
        Ok(())
    }

    pub fn is_mouse_visible(&self) -> bool {
        self.sdl.mouse().is_cursor_showing()
    }

    pub fn swap_buffers(&self) {
        self.sdl_window.gl_swap_window();
    }

    pub fn get_gamepad_name(&self, platform_id: i32) -> String {
        self.controllers[&platform_id].controller.name()
    }

    pub fn is_gamepad_vibration_supported(&self, platform_id: i32) -> bool {
        self.controllers[&platform_id].haptic.is_some()
    }

    pub fn set_gamepad_vibration(&mut self, platform_id: i32, strength: f32) {
        self.start_gamepad_vibration(platform_id, strength, SDL_HAPTIC_INFINITY);
    }

    pub fn start_gamepad_vibration(&mut self, platform_id: i32, strength: f32, duration: u32) {
        if let Some(haptic) = self
            .controllers
            .get_mut(&platform_id)
            .and_then(|c| c.haptic.as_mut())
        {
            haptic.rumble_play(strength, duration);
        }
    }

    pub fn stop_gamepad_vibration(&mut self, platform_id: i32) {
        if let Some(haptic) = self
            .controllers
            .get_mut(&platform_id)
            .and_then(|c| c.haptic.as_mut())
        {
            haptic.rumble_stop();
        }
    }
}

pub fn handle_events<S>(ctx: &mut Context, state: &mut S) -> Result
where
    S: State,
{
    while let Some(event) = ctx.window.event_pump.poll_event() {
        match event {
            SdlEvent::Quit { .. } => ctx.running = false, // TODO: Add a way to override this

            SdlEvent::Window { win_event, .. } => match win_event {
                WindowEvent::SizeChanged(width, height) => {
                    ctx.window.window_width = width;
                    ctx.window.window_height = height;

                    graphics::update_window_projection(ctx, width, height);
                    state.event(ctx, Event::Resized { width, height })?;
                }

                WindowEvent::FocusGained => {
                    state.event(ctx, Event::FocusGained)?;
                }

                WindowEvent::FocusLost => {
                    state.event(ctx, Event::FocusLost)?;
                }

                _ => {}
            },

            SdlEvent::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => {
                if let SdlKey::Escape = k {
                    if ctx.quit_on_escape {
                        ctx.running = false;
                    }
                }

                if let Some(key) = into_key(k) {
                    input::set_key_down(ctx, key);
                    state.event(ctx, Event::KeyPressed { key })?;
                }
            }

            SdlEvent::KeyUp {
                keycode: Some(k), ..
            } => {
                if let Some(key) = into_key(k) {
                    // TODO: This can cause some inputs to be missed at low tick rates.
                    // Could consider buffering input releases like Otter2D does?
                    input::set_key_up(ctx, key);
                    state.event(ctx, Event::KeyReleased { key })?;
                }
            }

            SdlEvent::MouseButtonDown { mouse_btn, .. } => {
                if let Some(button) = into_mouse_button(mouse_btn) {
                    input::set_mouse_button_down(ctx, button);
                    state.event(ctx, Event::MouseButtonPressed { button })?;
                }
            }

            SdlEvent::MouseButtonUp { mouse_btn, .. } => {
                if let Some(button) = into_mouse_button(mouse_btn) {
                    input::set_mouse_button_up(ctx, button);
                    state.event(ctx, Event::MouseButtonReleased { button })?;
                }
            }

            SdlEvent::MouseMotion { x, y, .. } => {
                let position = Vec2::new(x as f32, y as f32);
                input::set_mouse_position(ctx, position);
                state.event(ctx, Event::MouseMoved { position })?;
            }

            SdlEvent::TextInput { text, .. } => {
                input::push_text_input(ctx, &text);
                state.event(ctx, Event::TextInput { text })?;
            }

            SdlEvent::ControllerDeviceAdded { which, .. } => {
                let controller = ctx
                    .window
                    .controller_sys
                    .open(which)
                    .map_err(|e| TetraError::PlatformError(e.to_string()))?;

                let haptic = ctx.window.haptic_sys.open_from_joystick_id(which).ok();

                let platform_id = controller.instance_id();

                ctx.window
                    .controllers
                    .insert(platform_id, SdlController { controller, haptic });

                let id = input::add_gamepad(ctx, platform_id);

                state.event(ctx, Event::GamepadAdded { id })?;
            }

            SdlEvent::ControllerDeviceRemoved { which, .. } => {
                ctx.window.controllers.remove(&which).unwrap();
                let id = input::remove_gamepad(ctx, which);

                state.event(ctx, Event::GamepadRemoved { id })?;
            }

            SdlEvent::ControllerButtonDown { which, button, .. } => {
                if let Some(id) = input::map_gamepad_id(ctx, which) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, id) {
                        let button = button.into();

                        pad.set_button_down(button);
                        state.event(ctx, Event::GamepadButtonPressed { id, button })?;
                    }
                }
            }

            SdlEvent::ControllerButtonUp { which, button, .. } => {
                if let Some(id) = input::map_gamepad_id(ctx, which) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, id) {
                        let button = button.into();

                        // TODO: This can cause some inputs to be missed at low tick rates.
                        // Could consider buffering input releases like Otter2D does?
                        pad.set_button_up(button);
                        state.event(ctx, Event::GamepadButtonReleased { id, button })?;
                    }
                }
            }

            SdlEvent::ControllerAxisMotion {
                which, axis, value, ..
            } => {
                if let Some(id) = input::map_gamepad_id(ctx, which) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, id) {
                        let axis = axis.into();
                        let mapped_value = f32::from(value) / 32767.0;

                        pad.set_axis_position(axis, mapped_value);

                        let button = match axis {
                            GamepadAxis::LeftTrigger => Some(GamepadButton::LeftTrigger),
                            GamepadAxis::RightTrigger => Some(GamepadButton::RightTrigger),
                            _ => None,
                        };

                        if let Some(button) = button {
                            if value > 0 {
                                let pressed = pad.set_button_down(button);

                                if pressed {
                                    state.event(ctx, Event::GamepadButtonPressed { id, button })?;
                                }
                            } else {
                                let released = pad.set_button_up(button);

                                if released {
                                    state
                                        .event(ctx, Event::GamepadButtonReleased { id, button })?;
                                }
                            }
                        }

                        state.event(
                            ctx,
                            Event::GamepadAxisMoved {
                                id,
                                axis,
                                position: mapped_value,
                            },
                        )?;

                        let stick = match axis {
                            GamepadAxis::LeftStickX | GamepadAxis::LeftStickY => {
                                Some(GamepadStick::LeftStick)
                            }
                            GamepadAxis::RightStickX | GamepadAxis::RightStickY => {
                                Some(GamepadStick::RightStick)
                            }
                            _ => None,
                        };

                        if let Some(stick) = stick {
                            state.event(
                                ctx,
                                Event::GamepadStickMoved {
                                    id,
                                    stick,
                                    position: input::get_gamepad_stick_position(ctx, id, stick),
                                },
                            )?;
                        }
                    }
                }
            }

            _ => {}
        }
    }

    Ok(())
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
