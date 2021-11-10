// TODO: This file is getting way too huge.
use std::path::PathBuf;
use std::result;

use glow::Context as GlowContext;
use hashbrown::HashMap;
use sdl2::controller::{Axis as SdlGamepadAxis, Button as SdlGamepadButton, GameController};
use sdl2::event::{Event as SdlEvent, WindowEvent};
use sdl2::keyboard::{Keycode, Mod, Scancode};
use sdl2::mouse::{MouseButton as SdlMouseButton, MouseWheelDirection};
use sdl2::pixels::PixelMasks;
use sdl2::surface::Surface;
use sdl2::sys::SDL_WINDOWPOS_CENTERED_MASK;
use sdl2::video::{
    FullscreenType, GLContext as SdlGlContext, GLProfile, SwapInterval, Window as SdlWindow,
    WindowPos,
};
use sdl2::{EventPump, GameControllerSubsystem, JoystickSubsystem, Sdl, VideoSubsystem};

use crate::error::{Result, TetraError};
use crate::graphics::{self, ImageData};
use crate::input::{
    self, GamepadAxis, GamepadButton, GamepadStick, Key, KeyLabel, KeyModifierState, MouseButton,
};
use crate::math::Vec2;
use crate::window::WindowPosition;
use crate::{Context, ContextBuilder, Event, State};

struct SdlController {
    controller: GameController,
    slot: usize,
    supports_rumble: bool,
}

pub struct Window {
    sdl: Sdl,
    sdl_window: SdlWindow,

    event_pump: EventPump,
    video_sys: VideoSubsystem,
    controller_sys: GameControllerSubsystem,
    _joystick_sys: JoystickSubsystem,
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

        let _ = video_sys.gl_set_swap_interval(if settings.vsync {
            SwapInterval::VSync
        } else {
            SwapInterval::Immediate
        });

        let window = Window {
            sdl,
            sdl_window,

            event_pump,
            video_sys,
            controller_sys,
            _joystick_sys: joystick_sys,
            _gl_sys: gl_sys,

            controllers: HashMap::new(),

            window_visible: false,

            key_repeat: settings.key_repeat,
        };

        Ok((window, gl_ctx, window_width, window_height))
    }

    pub fn maximize(&mut self) {
        self.sdl_window.maximize();
    }

    pub fn minimize(&mut self) {
        self.sdl_window.minimize();
    }

    pub fn restore(&mut self) {
        self.sdl_window.restore();
    }

    pub fn focus(&mut self) {
        self.sdl_window.raise()
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

    pub fn set_minimum_size(&mut self, width: i32, height: i32) -> Result {
        self.sdl_window
            .set_minimum_size(width as u32, height as u32)
            .map_err(|e| TetraError::PlatformError(e.to_string()))
    }

    pub fn get_minimum_size(&self) -> (i32, i32) {
        let (width, height) = self.sdl_window.minimum_size();
        (width as i32, height as i32)
    }

    pub fn set_maximum_size(&mut self, width: i32, height: i32) -> Result {
        self.sdl_window
            .set_maximum_size(width as u32, height as u32)
            .map_err(|e| TetraError::PlatformError(e.to_string()))
    }

    pub fn get_maximum_size(&self) -> (i32, i32) {
        let (width, height) = self.sdl_window.maximum_size();
        (width as i32, height as i32)
    }

    pub fn set_position(&mut self, x: WindowPosition, y: WindowPosition) {
        self.sdl_window.set_position(x.into(), y.into());
    }

    pub fn get_position(&self) -> (i32, i32) {
        self.sdl_window.position()
    }

    pub fn set_decorated(&mut self, bordered: bool) {
        self.sdl_window.set_bordered(bordered);
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
        self.controllers
            .get(&platform_id)
            .map(|c| c.supports_rumble)
            .unwrap_or(false)
    }

    pub fn set_gamepad_vibration(&mut self, platform_id: u32, strength: f32) {
        self.start_gamepad_vibration(platform_id, strength, 0);
    }

    pub fn start_gamepad_vibration(&mut self, platform_id: u32, strength: f32, duration: u32) {
        if let Some(controller) = self
            .controllers
            .get_mut(&platform_id)
            .map(|c| &mut c.controller)
        {
            let int_strength = ((u16::MAX as f32) * strength) as u16;

            let _ = controller.set_rumble(int_strength, int_strength, duration);
        }
    }

    pub fn stop_gamepad_vibration(&mut self, platform_id: u32) {
        if let Some(controller) = self
            .controllers
            .get_mut(&platform_id)
            .map(|c| &mut c.controller)
        {
            let _ = controller.set_rumble(0, 0, 0);
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

    pub fn get_key_with_label(&self, key_label: KeyLabel) -> Option<Key> {
        let sdl_keycode = into_sdl_keycode(key_label);
        let sdl_scancode = Scancode::from_keycode(sdl_keycode)?;
        from_sdl_scancode(sdl_scancode)
    }

    pub fn get_key_label(&self, key: Key) -> Option<KeyLabel> {
        let sdl_scancode = into_sdl_scancode(key);
        let sdl_keycode = Keycode::from_scancode(sdl_scancode)?;
        from_sdl_keycode(sdl_keycode)
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
                keymod,
                ..
            } => {
                if !repeat || ctx.window.is_key_repeat_enabled() {
                    input::set_key_modifier_state(ctx, from_sdl_keymod(keymod));

                    if let Scancode::Escape = scancode {
                        if ctx.quit_on_escape {
                            ctx.running = false;
                        }
                    }

                    if let Some(key) = from_sdl_scancode(scancode) {
                        input::set_key_down(ctx, key);
                        state.event(ctx, Event::KeyPressed { key })?;
                    }
                }
            }

            SdlEvent::KeyUp {
                scancode: Some(scancode),
                keymod,
                ..
            } => {
                input::set_key_modifier_state(ctx, from_sdl_keymod(keymod));

                if let Some(key) = from_sdl_scancode(scancode) {
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
                let mut controller = ctx
                    .window
                    .controller_sys
                    .open(which)
                    .map_err(|e| TetraError::PlatformError(e.to_string()))?;

                let id = controller.instance_id();
                let slot = input::add_gamepad(ctx, id);

                let supports_rumble = controller.set_rumble(0, 0, 0).is_ok();

                ctx.window.controllers.insert(
                    id,
                    SdlController {
                        controller,
                        slot,
                        supports_rumble,
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
                        if let Some(button) = into_gamepad_button(button) {
                            pad.set_button_down(button);
                            state.event(ctx, Event::GamepadButtonPressed { id: slot, button })?;
                        }
                    }
                }
            }

            SdlEvent::ControllerButtonUp { which, button, .. } => {
                if let Some(slot) = ctx.window.controllers.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        if let Some(button) = into_gamepad_button(button) {
                            // TODO: This can cause some inputs to be missed at low tick rates.
                            // Could consider buffering input releases like Otter2D does?
                            pad.set_button_up(button);
                            state.event(ctx, Event::GamepadButtonReleased { id: slot, button })?;
                        }
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

macro_rules! key_mappings {
    (
        both {
            $($sdl_both:ident => $tetra_both:ident),*$(,)?
        }

        scancodes {
            $($sdl_scancode:ident => $tetra_key:ident),*$(,)?
        }

        keycodes {
            $($sdl_keycode:ident => $tetra_key_label:ident),*$(,)?
        }
    ) => {
        fn from_sdl_scancode(scancode: Scancode) -> Option<Key> {
            match scancode {
                $(
                    Scancode::$sdl_both => Some(Key::$tetra_both),
                )*

                $(
                    Scancode::$sdl_scancode => Some(Key::$tetra_key),
                )*

                _ => None,
            }
        }

        fn into_sdl_scancode(key: Key) -> Scancode {
            match key {
                $(
                    Key::$tetra_both => Scancode::$sdl_both,
                )*

                $(
                    Key::$tetra_key => Scancode::$sdl_scancode,
                )*
            }
        }

        fn from_sdl_keycode(keycode: Keycode) -> Option<KeyLabel> {
            match keycode {
                $(
                    Keycode::$sdl_both => Some(KeyLabel::$tetra_both),
                )*

                $(
                    Keycode::$sdl_keycode => Some(KeyLabel::$tetra_key_label),
                )*

                _ => None,
            }
        }

        fn into_sdl_keycode(key_label: KeyLabel) -> Keycode {
            match key_label {
                $(
                    KeyLabel::$tetra_both => Keycode::$sdl_both,
                )*

                $(
                    KeyLabel::$tetra_key_label => Keycode::$sdl_keycode,
                )*
            }
        }

    };
}

key_mappings! {
    both {
        A => A,
        B => B,
        C => C,
        D => D,
        E => E,
        F => F,
        G => G,
        H => H,
        I => I,
        J => J,
        K => K,
        L => L,
        M => M,
        N => N,
        O => O,
        P => P,
        Q => Q,
        R => R,
        S => S,
        T => T,
        U => U,
        V => V,
        W => W,
        X => X,
        Y => Y,
        Z => Z,

        Num0 => Num0,
        Num1 => Num1,
        Num2 => Num2,
        Num3 => Num3,
        Num4 => Num4,
        Num5 => Num5,
        Num6 => Num6,
        Num7 => Num7,
        Num8 => Num8,
        Num9 => Num9,

        F1 => F1,
        F2 => F2,
        F3 => F3,
        F4 => F4,
        F5 => F5,
        F6 => F6,
        F7 => F7,
        F8 => F8,
        F9 => F9,
        F10 => F10,
        F11 => F11,
        F12 => F12,
        F13 => F13,
        F14 => F14,
        F15 => F15,
        F16 => F16,
        F17 => F17,
        F18 => F18,
        F19 => F19,
        F20 => F20,
        F21 => F21,
        F22 => F22,
        F23 => F23,
        F24 => F24,

        NumLockClear => NumLock,
        Kp1 => NumPad1,
        Kp2 => NumPad2,
        Kp3 => NumPad3,
        Kp4 => NumPad4,
        Kp5 => NumPad5,
        Kp6 => NumPad6,
        Kp7 => NumPad7,
        Kp8 => NumPad8,
        Kp9 => NumPad9,
        Kp0 => NumPad0,
        KpPlus => NumPadPlus,
        KpMinus => NumPadMinus,
        KpMultiply => NumPadMultiply,
        KpDivide => NumPadDivide,
        KpEnter => NumPadEnter,

        LCtrl => LeftCtrl,
        LShift => LeftShift,
        LAlt => LeftAlt,
        RCtrl => RightCtrl,
        RShift => RightShift,
        RAlt => RightAlt,

        Up => Up,
        Down => Down,
        Left => Left,
        Right => Right,

        Backslash => Backslash,
        Backspace => Backspace,
        CapsLock => CapsLock,
        Comma => Comma,
        Delete => Delete,
        End => End,
        Return => Enter,
        Equals => Equals,
        Escape => Escape,
        Home => Home,
        Insert => Insert,
        LeftBracket => LeftBracket,
        Minus => Minus,
        PageDown => PageDown,
        PageUp => PageUp,
        Pause => Pause,
        Period => Period,
        PrintScreen => PrintScreen,
        RightBracket => RightBracket,
        ScrollLock => ScrollLock,
        Semicolon => Semicolon,
        Slash => Slash,
        Space => Space,
        Tab => Tab,
    }

    scancodes {
        Apostrophe => Quote,
        Grave => Backquote,
    }

    keycodes {
        Ampersand => Ampersand,
        Asterisk => Asterisk,
        At => At,
        Backquote => Backquote,
        Caret => Caret,
        Colon => Colon,
        Dollar => Dollar,
        Quotedbl => DoubleQuote,
        Exclaim => Exclaim,
        Greater => GreaterThan,
        Hash => Hash,
        LeftParen => LeftParen,
        Less => LessThan,
        Percent => Percent,
        Plus => Plus,
        Question => Question,
        Quote => Quote,
        RightParen => RightParen,
        Underscore => Underscore,
    }
}

fn from_sdl_keymod(keymod: Mod) -> KeyModifierState {
    KeyModifierState {
        ctrl: keymod.intersects(Mod::LCTRLMOD | Mod::RCTRLMOD),
        alt: keymod.intersects(Mod::LALTMOD | Mod::RALTMOD),
        shift: keymod.intersects(Mod::LSHIFTMOD | Mod::RSHIFTMOD),
    }
}

fn into_gamepad_button(button: SdlGamepadButton) -> Option<GamepadButton> {
    match button {
        SdlGamepadButton::A => Some(GamepadButton::A),
        SdlGamepadButton::B => Some(GamepadButton::B),
        SdlGamepadButton::X => Some(GamepadButton::X),
        SdlGamepadButton::Y => Some(GamepadButton::Y),
        SdlGamepadButton::DPadUp => Some(GamepadButton::Up),
        SdlGamepadButton::DPadDown => Some(GamepadButton::Down),
        SdlGamepadButton::DPadLeft => Some(GamepadButton::Left),
        SdlGamepadButton::DPadRight => Some(GamepadButton::Right),
        SdlGamepadButton::LeftShoulder => Some(GamepadButton::LeftShoulder),
        SdlGamepadButton::LeftStick => Some(GamepadButton::LeftStick),
        SdlGamepadButton::RightShoulder => Some(GamepadButton::RightShoulder),
        SdlGamepadButton::RightStick => Some(GamepadButton::RightStick),
        SdlGamepadButton::Start => Some(GamepadButton::Start),
        SdlGamepadButton::Back => Some(GamepadButton::Back),
        SdlGamepadButton::Guide => Some(GamepadButton::Guide),
        _ => None,
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
impl From<WindowPosition> for WindowPos {
    fn from(pos: WindowPosition) -> Self {
        // This is a bit of a hack to work around the fact that sdl2-rs doesn't
        // expose 'SDL_WINDOWPOS_CENTERED_DISPLAY' at all.
        match pos {
            WindowPosition::Centered(display_index) => {
                WindowPos::Positioned((SDL_WINDOWPOS_CENTERED_MASK | display_index as u32) as i32)
            }
            WindowPosition::Positioned(value) => WindowPos::Positioned(value),
        }
    }
}
