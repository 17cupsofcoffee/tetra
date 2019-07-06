mod sdl;

pub use sdl::SdlPlatform as ActivePlatform;

use glow::Context as GlowContext;

use crate::error::Result;
use crate::{Context, ContextBuilder};

pub trait Platform: Sized {
    type GlContext: GlowContext;

    fn new(builder: &ContextBuilder<'_>) -> Result<(Self, Self::GlContext, i32, i32)>;

    fn handle_events(ctx: &mut Context) -> Result;

    fn show_window(&mut self);
    fn hide_window(&mut self);

    fn get_window_title(&self) -> &str;
    fn set_window_title<S>(&mut self, title: S)
    where
        S: AsRef<str>;

    fn get_window_size(&self) -> (i32, i32);
    fn set_window_size(&mut self, width: i32, height: i32);

    fn toggle_fullscreen(&mut self) -> Result;
    fn enable_fullscreen(&mut self) -> Result;
    fn disable_fullscreen(&mut self) -> Result;
    fn is_fullscreen(&self) -> bool;

    fn set_mouse_visible(&mut self, mouse_visible: bool);
    fn is_mouse_visible(&self) -> bool;

    fn swap_buffers(&self);

    fn get_gamepad_name(&self, platform_id: i32) -> String;
    fn is_gamepad_vibration_supported(&self, platform_id: i32) -> bool;
    fn set_gamepad_vibration(&mut self, platform_id: i32, strength: f32);
    fn start_gamepad_vibration(&mut self, platform_id: i32, strength: f32, duration: u32);
    fn stop_gamepad_vibration(&mut self, platform_id: i32);
}
