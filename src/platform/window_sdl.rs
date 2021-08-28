// TODO: This file is getting way too huge.
use std::path::PathBuf;
use std::result;

use glow::Context as GlowContext;
use hashbrown::HashMap;
use sdl2::controller::{Axis as SdlGamepadAxis, Button as SdlGamepadButton, GameController};
use sdl2::event::{Event as SdlEvent, WindowEvent};
use sdl2::haptic::Haptic;
use sdl2::keyboard::Scancode;
use sdl2::mouse::{MouseButton as SdlMouseButton, MouseWheelDirection};
use sdl2::pixels::PixelMasks;
use sdl2::surface::Surface;
use sdl2::sys::SDL_HAPTIC_INFINITY;
use sdl2::video::{
    FullscreenType, GLContext as SdlGlContext, GLProfile, SwapInterval, Window as SdlWindow,
};
use sdl2::{
    EventPump, GameControllerSubsystem, HapticSubsystem, JoystickSubsystem, Sdl, VideoSubsystem,
};

use crate::error::{Result, TetraError};
use crate::graphics::{self, ImageData};
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

        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);
        gl_attr.set_red_size(8);
        gl_attr.set_green_size(8);
        gl_attr.set_blue_size(8);
        gl_attr.set_alpha_size(8);
        gl_attr.set_double_buffer(true);

        if settings.multisampling > 0 {
            gl_attr.set_multisample_buffers(1);
            gl_attr.set_multisample_samples(settings.multisampling);
        }

        if settings.stencil_buffer {
            gl_attr.set_stencil_size(8);
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

    pub fn set_icon(&mut self, data: &mut ImageData) -> Result {
        let (width, height) = data.size();

        let surface = Surface::from_data_pixelmasks(
            data.as_mut_bytes(),
            width as u32,
            height as u32,
            width as u32 * 4,
            PixelMasks {
                bpp: 32,
                rmask: 0x000000FF,
                gmask: 0x0000FF00,
                bmask: 0x00FF0000,
                amask: 0xFF000000,
            },
        )
        .map_err(TetraError::PlatformError)?;

        self.sdl_window.set_icon(surface);

        Ok(())
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
                scancode: Some(scancode),
                repeat,
                ..
            } => {
                if !repeat || ctx.window.is_key_repeat_enabled() {
                    if let Scancode::Escape = scancode {
                        if ctx.quit_on_escape {
                            ctx.running = false;
                        }
                    }

                    if let Some(key) = into_key(scancode) {
                        input::set_key_down(ctx, key);
                        state.event(ctx, Event::KeyPressed { key })?;
                    }
                }
            }

            SdlEvent::KeyUp {
                scancode: Some(scancode),
                ..
            } => {
                if let Some(key) = into_key(scancode) {
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

fn into_key(scancode: Scancode) -> Option<Key> {
    match scancode {
        Scancode::A => Some(Key::A),
        Scancode::B => Some(Key::B),
        Scancode::C => Some(Key::C),
        Scancode::D => Some(Key::D),
        Scancode::E => Some(Key::E),
        Scancode::F => Some(Key::F),
        Scancode::G => Some(Key::G),
        Scancode::H => Some(Key::H),
        Scancode::I => Some(Key::I),
        Scancode::J => Some(Key::J),
        Scancode::K => Some(Key::K),
        Scancode::L => Some(Key::L),
        Scancode::M => Some(Key::M),
        Scancode::N => Some(Key::N),
        Scancode::O => Some(Key::O),
        Scancode::P => Some(Key::P),
        Scancode::Q => Some(Key::Q),
        Scancode::R => Some(Key::R),
        Scancode::S => Some(Key::S),
        Scancode::T => Some(Key::T),
        Scancode::U => Some(Key::U),
        Scancode::V => Some(Key::V),
        Scancode::W => Some(Key::W),
        Scancode::X => Some(Key::X),
        Scancode::Y => Some(Key::Y),
        Scancode::Z => Some(Key::Z),

        Scancode::Num0 => Some(Key::Num0),
        Scancode::Num1 => Some(Key::Num1),
        Scancode::Num2 => Some(Key::Num2),
        Scancode::Num3 => Some(Key::Num3),
        Scancode::Num4 => Some(Key::Num4),
        Scancode::Num5 => Some(Key::Num5),
        Scancode::Num6 => Some(Key::Num6),
        Scancode::Num7 => Some(Key::Num7),
        Scancode::Num8 => Some(Key::Num8),
        Scancode::Num9 => Some(Key::Num9),

        Scancode::F1 => Some(Key::F1),
        Scancode::F2 => Some(Key::F2),
        Scancode::F3 => Some(Key::F3),
        Scancode::F4 => Some(Key::F4),
        Scancode::F5 => Some(Key::F5),
        Scancode::F6 => Some(Key::F6),
        Scancode::F7 => Some(Key::F7),
        Scancode::F8 => Some(Key::F8),
        Scancode::F9 => Some(Key::F9),
        Scancode::F10 => Some(Key::F10),
        Scancode::F11 => Some(Key::F11),
        Scancode::F12 => Some(Key::F12),
        Scancode::F13 => Some(Key::F13),
        Scancode::F14 => Some(Key::F14),
        Scancode::F15 => Some(Key::F15),
        Scancode::F16 => Some(Key::F16),
        Scancode::F17 => Some(Key::F17),
        Scancode::F18 => Some(Key::F18),
        Scancode::F19 => Some(Key::F19),
        Scancode::F20 => Some(Key::F20),
        Scancode::F21 => Some(Key::F21),
        Scancode::F22 => Some(Key::F22),
        Scancode::F23 => Some(Key::F23),
        Scancode::F24 => Some(Key::F24),

        Scancode::NumLockClear => Some(Key::NumLock),
        Scancode::Kp1 => Some(Key::NumPad1),
        Scancode::Kp2 => Some(Key::NumPad2),
        Scancode::Kp3 => Some(Key::NumPad3),
        Scancode::Kp4 => Some(Key::NumPad4),
        Scancode::Kp5 => Some(Key::NumPad5),
        Scancode::Kp6 => Some(Key::NumPad6),
        Scancode::Kp7 => Some(Key::NumPad7),
        Scancode::Kp8 => Some(Key::NumPad8),
        Scancode::Kp9 => Some(Key::NumPad9),
        Scancode::Kp0 => Some(Key::NumPad0),
        Scancode::KpPlus => Some(Key::NumPadPlus),
        Scancode::KpMinus => Some(Key::NumPadMinus),
        Scancode::KpMultiply => Some(Key::NumPadMultiply),
        Scancode::KpDivide => Some(Key::NumPadDivide),
        Scancode::KpEnter => Some(Key::NumPadEnter),

        Scancode::LCtrl => Some(Key::LeftCtrl),
        Scancode::LShift => Some(Key::LeftShift),
        Scancode::LAlt => Some(Key::LeftAlt),
        Scancode::RCtrl => Some(Key::RightCtrl),
        Scancode::RShift => Some(Key::RightShift),
        Scancode::RAlt => Some(Key::RightAlt),

        Scancode::Up => Some(Key::Up),
        Scancode::Down => Some(Key::Down),
        Scancode::Left => Some(Key::Left),
        Scancode::Right => Some(Key::Right),

        Scancode::Grave => Some(Key::Backquote),
        Scancode::Backslash => Some(Key::Backslash),
        Scancode::Backspace => Some(Key::Backspace),
        Scancode::CapsLock => Some(Key::CapsLock),
        Scancode::Comma => Some(Key::Comma),
        Scancode::Delete => Some(Key::Delete),
        Scancode::End => Some(Key::End),
        Scancode::Return => Some(Key::Enter),
        Scancode::Equals => Some(Key::Equals),
        Scancode::Escape => Some(Key::Escape),
        Scancode::Home => Some(Key::Home),
        Scancode::Insert => Some(Key::Insert),
        Scancode::LeftBracket => Some(Key::LeftBracket),
        Scancode::Minus => Some(Key::Minus),
        Scancode::PageDown => Some(Key::PageDown),
        Scancode::PageUp => Some(Key::PageUp),
        Scancode::Pause => Some(Key::Pause),
        Scancode::Period => Some(Key::Period),
        Scancode::PrintScreen => Some(Key::PrintScreen),
        Scancode::RightBracket => Some(Key::RightBracket),
        Scancode::ScrollLock => Some(Key::ScrollLock),
        Scancode::Semicolon => Some(Key::Semicolon),
        Scancode::Slash => Some(Key::Slash),
        Scancode::Space => Some(Key::Space),
        Scancode::Tab => Some(Key::Tab),

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
