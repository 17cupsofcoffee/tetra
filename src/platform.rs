//! The platform abstraction used for windowing, input and creating the GL context.
//!
//! All code interacting with SDL must be placed within this module. This is to facilitate
//! creating alternate backends in the future.
//!
//! The interface for this module is *not* stable, and will likely not be made public
//! in its current form.

use glow::native::Context as GlContext;
use hashbrown::HashMap;
use sdl2::controller::{Axis as SdlAxis, Button as SdlButton, GameController};
use sdl2::event::{Event, WindowEvent};
use sdl2::haptic::Haptic;
use sdl2::sys::SDL_HAPTIC_INFINITY;
use sdl2::video::{FullscreenType, GLContext as SdlGlContext, GLProfile, Window};
use sdl2::{GameControllerSubsystem, HapticSubsystem, JoystickSubsystem, Sdl, VideoSubsystem};

use crate::error::{Result, TetraError};
use crate::graphics::{self, Vec2};
use crate::input::{self, GamepadAxis, GamepadButton, Key};
use crate::window;
use crate::{Context, ContextBuilder};

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
    pub fn new(builder: &ContextBuilder<'_>) -> Result<(Platform, GlContext, i32, i32)> {
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
            video_sys.window(builder.title, window_width as u32, window_height as u32);

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
                if let Key::Escape = k {
                    if ctx.quit_on_escape {
                        ctx.running = false;
                    }
                }

                input::set_key_down(ctx, k);
            }

            Event::KeyUp {
                keycode: Some(k), ..
            } => {
                // TODO: This can cause some inputs to be missed at low tick rates.
                // Could consider buffering input releases like Otter2D does?
                input::set_key_up(ctx, k);
            }

            Event::MouseButtonDown { mouse_btn, .. } => {
                input::set_mouse_button_down(ctx, mouse_btn);
            }

            Event::MouseButtonUp { mouse_btn, .. } => {
                input::set_mouse_button_up(ctx, mouse_btn);
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
                            SdlAxis::TriggerLeft => {
                                if value > 0 {
                                    pad.set_button_down(GamepadButton::LeftTrigger);
                                } else {
                                    pad.set_button_up(GamepadButton::LeftTrigger);
                                }
                            }

                            SdlAxis::TriggerRight => {
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

pub fn show_window(ctx: &mut Context) {
    ctx.platform.window.show();
}

pub fn hide_window(ctx: &mut Context) {
    ctx.platform.window.hide();
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
