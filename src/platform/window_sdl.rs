// TODO: This file is getting way too huge.
use std::path::PathBuf;
use std::result;

use glow::Context as GlowContext;
use hashbrown::HashMap;
use sdl3::event::{DisplayEvent, Event as SdlEvent, WindowEvent};
use sdl3::gamepad::{Axis as SdlGamepadAxis, Button as SdlGamepadButton, Gamepad};
use sdl3::keyboard::{Keycode, Mod, Scancode};
use sdl3::mouse::{MouseButton as SdlMouseButton, MouseWheelDirection};
use sdl3::pixels::PixelMasks;
use sdl3::surface::Surface;
use sdl3::sys::keycode::SDL_KMOD_NONE;
use sdl3::sys::video::SDL_WINDOWPOS_CENTERED_MASK;
use sdl3::video::{
    Display, FullscreenType, GLContext as SdlGlContext, GLProfile, SwapInterval,
    Window as SdlWindow, WindowBuildError, WindowPos,
};
use sdl3::{EventPump, GamepadSubsystem, IntegerOrSdlError, Sdl, VideoSubsystem};

use crate::error::{Result, TetraError};
use crate::graphics::{self, ImageData};
use crate::input::{
    self, GamepadAxis, GamepadButton, GamepadStick, Key, KeyLabel, KeyModifierState, MouseButton,
};
use crate::math::Vec2;
use crate::window::WindowPosition;
use crate::{Context, ContextBuilder, Event, State};

struct SdlGamepad {
    gamepad: Gamepad,
    slot: usize,
    supports_rumble: bool,
}

pub struct Window {
    sdl: Sdl,
    sdl_window: SdlWindow,

    event_pump: EventPump,
    video_sys: VideoSubsystem,
    gamepad_sys: GamepadSubsystem,
    _gl_sys: SdlGlContext,

    gamepads: HashMap<u32, SdlGamepad>,
    displays: Vec<Display>,

    window_visible: bool,

    key_repeat: bool,
}

impl Window {
    pub fn new(settings: &ContextBuilder) -> Result<(Window, GlowContext, i32, i32)> {
        let sdl = sdl3::init()?;
        let event_pump = sdl.event_pump()?;
        let video_sys = sdl.video()?;
        let gamepad_sys = sdl.gamepad()?;

        sdl3::hint::set("SDL_JOYSTICK_ALLOW_BACKGROUND_EVENTS", "1");

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
            window_builder.high_pixel_density();
        }

        if settings.grab_mouse {
            window_builder.input_grabbed();
        }

        let mut sdl_window = window_builder.build()?;

        if settings.maximized {
            sdl_window.maximize();
        } else if settings.minimized {
            sdl_window.minimize();
        }

        if settings.fullscreen {
            sdl_window.set_fullscreen(true)?;
        }

        let size = sdl_window.size_in_pixels();
        let window_width = size.0 as i32;
        let window_height = size.1 as i32;

        let displays = video_sys.displays()?;

        let gl_sys = sdl_window.gl_create_context()?;

        let gl_ctx = unsafe {
            GlowContext::from_loader_function(|s| {
                if let Some(ptr) = video_sys.gl_get_proc_address(s) {
                    ptr as *const _
                } else {
                    std::ptr::null()
                }
            })
        };

        let _ = video_sys.gl_set_swap_interval(if settings.vsync {
            SwapInterval::VSync
        } else {
            SwapInterval::Immediate
        });

        sdl.mouse()
            .set_relative_mouse_mode(&sdl_window, settings.relative_mouse_mode);

        sdl.mouse().show_cursor(settings.show_mouse);

        let window = Window {
            sdl,
            sdl_window,

            event_pump,
            video_sys,
            gamepad_sys,
            _gl_sys: gl_sys,

            gamepads: HashMap::new(),
            displays,

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
        self.sdl_window.raise();
    }

    pub fn get_refresh_rate(&self) -> Result<f32> {
        let refresh_rate = self
            .sdl_window
            .get_display()
            .and_then(|d| d.get_mode())
            .map(|display_mode| display_mode.refresh_rate)?;

        Ok(refresh_rate)
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
        let (width, height) = self.sdl_window.size_in_pixels();
        (width as i32, height as i32)
    }

    pub fn set_window_size(&mut self, width: i32, height: i32) -> Result {
        self.sdl_window.set_size(width as u32, height as u32)?;

        Ok(())
    }

    pub fn set_minimum_size(&mut self, width: i32, height: i32) -> Result {
        self.sdl_window
            .set_minimum_size(width as u32, height as u32)?;

        Ok(())
    }

    pub fn get_minimum_size(&self) -> (i32, i32) {
        let (width, height) = self.sdl_window.minimum_size();
        (width as i32, height as i32)
    }

    pub fn set_maximum_size(&mut self, width: i32, height: i32) -> Result {
        self.sdl_window
            .set_maximum_size(width as u32, height as u32)?;

        Ok(())
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
            &PixelMasks {
                bpp: 32,
                rmask: 0x000000FF,
                gmask: 0x0000FF00,
                bmask: 0x00FF0000,
                amask: 0xFF000000,
            },
        )?;

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
            self.sdl_window.hide();
        }

        self.window_visible = visible;
    }

    pub fn get_dpi_scale(&self) -> f32 {
        self.sdl_window.display_scale()
    }

    pub fn get_monitor_count(&self) -> usize {
        self.displays.len()
    }

    pub fn get_monitor(&self, monitor_index: usize) -> Result<&Display> {
        self.displays.get(monitor_index).ok_or_else(|| {
            TetraError::PlatformError(format!("invalid monitor index: {}", monitor_index))
        })
    }

    pub fn get_monitor_name(&self, monitor_index: usize) -> Result<String> {
        self.get_monitor(monitor_index).and_then(|m| {
            m.get_name()
                .map_err(|e| TetraError::PlatformError(e.to_string()))
        })
    }

    pub fn get_monitor_size(&self, monitor_index: usize) -> Result<(i32, i32)> {
        let bounds = self.get_monitor(monitor_index).and_then(|m| {
            m.get_bounds()
                .map_err(|e| TetraError::PlatformError(e.to_string()))
        })?;

        Ok((bounds.w, bounds.h))
    }

    pub fn get_current_monitor(&self) -> Result<usize> {
        let display = self.sdl_window.get_display()?;

        for (i, d) in self.displays.iter().enumerate() {
            if d == &display {
                return Ok(i);
            }
        }

        Err(TetraError::PlatformError(
            "could not find current monitor".into(),
        ))
    }

    pub fn set_vsync(&mut self, vsync: bool) -> Result {
        self.video_sys.gl_set_swap_interval(if vsync {
            SwapInterval::VSync
        } else {
            SwapInterval::Immediate
        })?;

        Ok(())
    }

    pub fn is_vsync_enabled(&self) -> Result<bool> {
        let vsync = self
            .video_sys
            .gl_get_swap_interval()
            .map(|s| s != SwapInterval::Immediate)?;

        Ok(vsync)
    }

    pub fn set_fullscreen(&mut self, fullscreen: bool) -> Result {
        self.sdl_window
            .set_fullscreen(fullscreen)
            .map_err(|e| TetraError::PlatformError(e.to_string()))?;

        let (width, height) = self.sdl_window.size_in_pixels();

        self.set_window_size(width as i32, height as i32)?;

        Ok(())
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
        self.sdl_window.set_mouse_grab(mouse_grabbed);
    }

    pub fn is_mouse_grabbed(&self) -> bool {
        self.sdl_window.mouse_grab()
    }

    pub fn set_relative_mouse_mode(&mut self, relative_mouse_mode: bool) {
        self.sdl
            .mouse()
            .set_relative_mouse_mode(&self.sdl_window, relative_mouse_mode);
    }

    pub fn is_relative_mouse_mode(&self) -> bool {
        self.sdl.mouse().relative_mouse_mode(&self.sdl_window)
    }

    pub fn get_clipboard_text(&self) -> Result<String> {
        let clipboard_text = self.video_sys.clipboard().clipboard_text()?;

        Ok(clipboard_text)
    }

    pub fn set_clipboard_text(&self, text: &str) -> Result {
        self.video_sys.clipboard().set_clipboard_text(text)?;

        Ok(())
    }

    pub fn swap_buffers(&self) {
        self.sdl_window.gl_swap_window();
    }

    pub fn get_gamepad_name(&self, platform_id: u32) -> Option<String> {
        self.gamepads[&platform_id].gamepad.name()
    }

    pub fn is_gamepad_vibration_supported(&self, platform_id: u32) -> bool {
        self.gamepads
            .get(&platform_id)
            .map(|c| c.supports_rumble)
            .unwrap_or(false)
    }

    pub fn set_gamepad_vibration(&mut self, platform_id: u32, strength: f32) {
        self.start_gamepad_vibration(platform_id, strength, 0);
    }

    pub fn start_gamepad_vibration(&mut self, platform_id: u32, strength: f32, duration: u32) {
        if let Some(gamepad) = self.gamepads.get_mut(&platform_id).map(|c| &mut c.gamepad) {
            let int_strength = ((u16::MAX as f32) * strength) as u16;

            let _ = gamepad.set_rumble(int_strength, int_strength, duration);
        }
    }

    pub fn stop_gamepad_vibration(&mut self, platform_id: u32) {
        if let Some(gamepad) = self.gamepads.get_mut(&platform_id).map(|c| &mut c.gamepad) {
            let _ = gamepad.set_rumble(0, 0, 0);
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
        let sdl_scancode = Scancode::from_keycode(sdl_keycode, std::ptr::null_mut())?;
        from_sdl_scancode(sdl_scancode)
    }

    pub fn get_key_label(&self, key: Key) -> Option<KeyLabel> {
        let sdl_scancode = into_sdl_scancode(key);
        let sdl_keycode = Keycode::from_scancode(sdl_scancode, SDL_KMOD_NONE, false)?;
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
                WindowEvent::PixelSizeChanged(width, height) => {
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

            SdlEvent::Display {
                display_event: DisplayEvent::Added | DisplayEvent::Removed | DisplayEvent::Moved,
                ..
            } => {
                ctx.window.displays = ctx
                    .window
                    .video_sys
                    .displays()
                    .map_err(|e| TetraError::PlatformError(e.to_string()))?;
            }

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
                let position = Vec2::new(x, y);
                let delta = Vec2::new(xrel, yrel);

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
                let mut gamepad = ctx
                    .window
                    .gamepad_sys
                    .open(which)
                    .map_err(|e| TetraError::PlatformError(e.to_string()))?;

                let slot = input::add_gamepad(ctx, which);

                let supports_rumble = gamepad.set_rumble(0, 0, 0).is_ok();

                ctx.window.gamepads.insert(
                    which,
                    SdlGamepad {
                        gamepad,
                        slot,
                        supports_rumble,
                    },
                );

                state.event(ctx, Event::GamepadAdded { id: slot })?;
            }

            SdlEvent::ControllerDeviceRemoved { which, .. } => {
                let gamepad = ctx.window.gamepads.remove(&which).unwrap();
                input::remove_gamepad(ctx, gamepad.slot);

                state.event(ctx, Event::GamepadRemoved { id: gamepad.slot })?;
            }

            SdlEvent::ControllerButtonDown { which, button, .. } => {
                if let Some(slot) = ctx.window.gamepads.get(&which).map(|c| c.slot) {
                    if let Some(pad) = input::get_gamepad_mut(ctx, slot) {
                        if let Some(button) = into_gamepad_button(button) {
                            pad.set_button_down(button);
                            state.event(ctx, Event::GamepadButtonPressed { id: slot, button })?;
                        }
                    }
                }
            }

            SdlEvent::ControllerButtonUp { which, button, .. } => {
                if let Some(slot) = ctx.window.gamepads.get(&which).map(|c| c.slot) {
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
                if let Some(slot) = ctx.window.gamepads.get(&which).map(|c| c.slot) {
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

        _0 => Num0,
        _1 => Num1,
        _2 => Num2,
        _3 => Num3,
        _4 => Num4,
        _5 => Num5,
        _6 => Num6,
        _7 => Num7,
        _8 => Num8,
        _9 => Num9,

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
        Grave => Backquote,
        Caret => Caret,
        Colon => Colon,
        Dollar => Dollar,
        DblApostrophe => DoubleQuote,
        Exclaim => Exclaim,
        Greater => GreaterThan,
        Hash => Hash,
        LeftParen => LeftParen,
        Less => LessThan,
        Percent => Percent,
        Plus => Plus,
        Question => Question,
        Apostrophe => Quote,
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
        SdlGamepadButton::South => Some(GamepadButton::A),
        SdlGamepadButton::East => Some(GamepadButton::B),
        SdlGamepadButton::West => Some(GamepadButton::X),
        SdlGamepadButton::North => Some(GamepadButton::Y),
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

impl From<sdl3::Error> for TetraError {
    fn from(error: sdl3::Error) -> Self {
        TetraError::PlatformError(error.to_string())
    }
}

impl From<WindowBuildError> for TetraError {
    fn from(error: WindowBuildError) -> Self {
        TetraError::PlatformError(error.to_string())
    }
}

impl From<IntegerOrSdlError> for TetraError {
    fn from(error: IntegerOrSdlError) -> Self {
        TetraError::PlatformError(error.to_string())
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
        // This is a bit of a hack to work around the fact that sdl3-rs doesn't
        // expose 'SDL_WINDOWPOS_CENTERED_DISPLAY' at all.
        match pos {
            WindowPosition::Centered(display_index) => {
                WindowPos::Positioned((SDL_WINDOWPOS_CENTERED_MASK | display_index as u32) as i32)
            }
            WindowPosition::Positioned(value) => WindowPos::Positioned(value),
        }
    }
}
