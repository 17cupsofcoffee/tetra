// TODO: This file is getting way too huge.
use std::path::PathBuf;
use std::result;

use glow::Context as GlowContext;
use hashbrown::HashMap;
use sdl2::controller::{Axis as SdlGamepadAxis, Button as SdlGamepadButton, GameController};
use sdl2::event::{Event as SdlEvent, WindowEvent};
use sdl2::haptic::Haptic;
use sdl2::keyboard::Keycode as SdlKey;
use sdl2::mouse::{MouseButton as SdlMouseButton, MouseWheelDirection};
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
    slot: usize,
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

    controllers: HashMap<u32, SdlController>,

    window_visible: bool,

    key_repeat: bool,
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
        gl_attr.set_stencil_size(8);
        gl_attr.set_double_buffer(true);

        if settings.multisampling > 0 {
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(settings.multisampling);
        }

        if settings.screen_saver_enabled {
            video_sys.enable_screen_saver();
        } else {
            video_sys.disable_screen_saver();
        }

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

        if settings.high_dpi {
            window_builder.allow_highdpi();
        }

        if settings.grab_mouse {
            window_builder.input_grabbed();
        }

        sdl.mouse()
            .set_relative_mouse_mode(settings.relative_mouse_mode);

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

        let gl_ctx = unsafe {
            GlowContext::from_loader_function(|s| video_sys.gl_get_proc_address(s) as *const _)
        };

        video_sys
            .gl_set_swap_interval(if settings.vsync {
                SwapInterval::VSync
            } else {
                SwapInterval::Immediate
            })
            .map_err(TetraError::FailedToChangeDisplayMode)?;

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

            window_visible: false,

            key_repeat: settings.key_repeat,
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

    pub fn get_window_size(&self) -> (i32, i32) {
        let (width, height) = self.sdl_window.size();
        (width as i32, height as i32)
    }

    pub fn get_physical_size(&self) -> (i32, i32) {
        let (width, height) = self.sdl_window.drawable_size();
        (width as i32, height as i32)
    }

    pub fn set_window_size(&mut self, width: i32, height: i32) -> Result {
        self.sdl_window
            .set_size(width as u32, height as u32)
            .map_err(|e| TetraError::FailedToChangeDisplayMode(e.to_string()))
    }

    pub fn is_visible(&self) -> bool {
        self.window_visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        if visible {
            self.sdl_window.show();
        } else {
            self.sdl_window.hide()
        }

        self.window_visible = visible;
    }

    pub fn get_dpi_scale(&self) -> f32 {
        self.sdl_window.drawable_size().0 as f32 / self.sdl_window.size().0 as f32
    }

    pub fn get_monitor_count(&self) -> Result<i32> {
        self.video_sys
            .num_video_displays()
            .map_err(TetraError::PlatformError)
    }

    pub fn get_monitor_name(&self, monitor_index: i32) -> Result<String> {
        self.video_sys
            .display_name(monitor_index)
            .map_err(TetraError::PlatformError)
    }

    pub fn get_monitor_size(&self, monitor_index: i32) -> Result<(i32, i32)> {
        let display_mode = self
            .video_sys
            .desktop_display_mode(monitor_index)
            .map_err(TetraError::PlatformError)?;

        Ok((display_mode.w, display_mode.h))
    }

    pub fn get_current_monitor(&self) -> Result<i32> {
        self.sdl_window
            .display_index()
            .map_err(TetraError::PlatformError)
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

    pub fn set_mouse_grabbed(&mut self, mouse_grabbed: bool) {
        self.sdl_window.set_grab(mouse_grabbed);
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.sdl_window.grab()
    }

    pub fn set_relative_mouse_mode(&mut self, relative_mouse_mode: bool) {
        self.sdl
            .mouse()
            .set_relative_mouse_mode(relative_mouse_mode);
    }

    pub fn is_relative_mouse_mode(&self) -> bool {
        self.sdl.mouse().relative_mouse_mode()
    }

    pub fn get_clipboard_text(&self) -> Result<String> {
        self.video_sys
            .clipboard()
            .clipboard_text()
            .map_err(TetraError::PlatformError)
    }

    pub fn set_clipboard_text(&self, text: &str) -> Result {
        self.video_sys
            .clipboard()
            .set_clipboard_text(text)
            .map_err(TetraError::PlatformError)
    }

    pub fn swap_buffers(&self) {
        self.sdl_window.gl_swap_window();
    }

    pub fn get_gamepad_name(&self, platform_id: u32) -> String {
        self.controllers[&platform_id].controller.name()
    }

    pub fn is_gamepad_vibration_supported(&self, platform_id: u32) -> bool {
        self.controllers[&platform_id].haptic.is_some()
    }

    pub fn set_gamepad_vibration(&mut self, platform_id: u32, strength: f32) {
        self.start_gamepad_vibration(platform_id, strength, SDL_HAPTIC_INFINITY);
    }

    pub fn start_gamepad_vibration(&mut self, platform_id: u32, strength: f32, duration: u32) {
        if let Some(haptic) = self
            .controllers
            .get_mut(&platform_id)
            .and_then(|c| c.haptic.as_mut())
        {
            haptic.rumble_play(strength, duration);
        }
    }

    pub fn stop_gamepad_vibration(&mut self, platform_id: u32) {
        if let Some(haptic) = self
            .controllers
            .get_mut(&platform_id)
            .and_then(|c| c.haptic.as_mut())
        {
            haptic.rumble_stop();
        }
    }

    pub fn set_screen_saver_enabled(&self, screen_saver_enabled: bool) {
        if screen_saver_enabled {
            self.video_sys.enable_screen_saver()
        } else {
            self.video_sys.disable_screen_saver()
        }
    }

    pub fn is_screen_saver_enabled(&self) -> bool {
        self.video_sys.is_screen_saver_enabled()
    }

    pub fn set_key_repeat_enabled(&mut self, key_repeat: bool) {
        self.key_repeat = key_repeat;
    }

    pub fn is_key_repeat_enabled(&self) -> bool {
        self.key_repeat
    }
}

pub fn handle_events<S, E>(ctx: &mut Context, state: &mut S) -> result::Result<(), E>
where
    S: State<E>,
    E: From<TetraError>,
{
    while let Some(event) = ctx.window.event_pump.poll_event() {
        match event {
            SdlEvent::Quit { .. } => ctx.running = false, // TODO: Add a way to override this

            SdlEvent::Window { win_event, .. } => match win_event {
                WindowEvent::SizeChanged(width, height) => {
                    graphics::set_viewport_size(ctx);
                    state.event(ctx, Event::Resized { width, height })?;
                }

                WindowEvent::Restored => {
                    state.event(ctx, Event::Restored)?;
                }

                WindowEvent::Minimized => {
                    state.event(ctx, Event::Minimized)?;
                }

                WindowEvent::Maximized => {
                    state.event(ctx, Event::Maximized)?;
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
                repeat,
                ..
            } => {
                if !repeat || ctx.window.is_key_repeat_enabled() {
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

            SdlEvent::MouseMotion {
                x, y, xrel, yrel, ..
            } => {
                let position = Vec2::new(x as f32, y as f32);
                let delta = Vec2::new(xrel as f32, yrel as f32);

                input::set_mouse_position(ctx, position);
                state.event(ctx, Event::MouseMoved { position, delta })?;
            }

            SdlEvent::MouseWheel {
                x, y, direction, ..
            } => {
                let amount = match direction {
                    MouseWheelDirection::Flipped => Vec2::new(-x, -y),
                    _ => Vec2::new(x, y),
                };

                input::apply_mouse_wheel_movement(ctx, amount);
                state.event(ctx, Event::MouseWheelMoved { amount })?
            }

            SdlEvent::TextInput { text, .. } => {
                input::push_text_input(ctx, &text);
                state.event(ctx, Event::TextInput { text })?;
            }

            SdlEvent::DropFile { filename, .. } => {
                state.event(
                    ctx,
                    Event::FileDropped {
                        path: PathBuf::from(filename),
                    },
                )?;
            }

            SdlEvent::ControllerDeviceAdded { which, .. } => {
                let controller = ctx
                    .window
                    .controller_sys
                    .open(which)
                    .map_err(|e| TetraError::PlatformError(e.to_string()))?;

                let haptic = ctx.window.haptic_sys.open_from_joystick_id(which).ok();
                let id = controller.instance_id();
                let slot = input::add_gamepad(ctx, id);

                ctx.window.controllers.insert(
                    id,
                    SdlController {
                        controller,
                        haptic,
                        slot,
                    },
                );

                state.event(ctx, Event::GamepadAdded { id: slot })?;
            }

            SdlEvent::ControllerDeviceRemoved { which, .. } => {
                let controller = ctx.window.controllers.remove(&which).unwrap();
                input::remove_gamepad(ctx, controller.slot);

                state.event(
                    ctx,
                    Event::GamepadRemoved {
                        id: controller.slot,
                    },
                )?;
            }

            SdlEvent::ControllerButtonDown { which, button, .. } => {
                if let Some(slot) = ctx.window.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        let button = button.into();

                        pad.set_button_down(button);
                        state.event(ctx, Event::GamepadButtonPressed { id: slot, button })?;
                    }
                }
            }

            SdlEvent::ControllerButtonUp { which, button, .. } => {
                if let Some(slot) = ctx.window.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        let button = button.into();

                        // TODO: This can cause some inputs to be missed at low tick rates.
                        // Could consider buffering input releases like Otter2D does?
                        pad.set_button_up(button);
                        state.event(ctx, Event::GamepadButtonReleased { id: slot, button })?;
                    }
                }
            }

            SdlEvent::ControllerAxisMotion {
                which, axis, value, ..
            } => {
                if let Some(slot) = ctx.window.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        let axis = axis.into();

                        let mapped_value = if value > 0 {
                            f32::from(value) / 32767.0
                        } else {
                            f32::from(value) / 32768.0
                        };

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
                                    state.event(
                                        ctx,
                                        Event::GamepadButtonPressed { id: slot, button },
                                    )?;
                                }
                            } else {
                                let released = pad.set_button_up(button);

                                if released {
                                    state.event(
                                        ctx,
                                        Event::GamepadButtonReleased { id: slot, button },
                                    )?;
                                }
                            }
                        }

                        state.event(
                            ctx,
                            Event::GamepadAxisMoved {
                                id: slot,
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
                                    id: slot,
                                    stick,
                                    position: input::get_gamepad_stick_position(ctx, slot, stick),
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
