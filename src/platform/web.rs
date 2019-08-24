use std::cell::RefCell;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::audio::{RemoteControls, Sound, SoundInstance};
use crate::error::{Result, TetraError};
use crate::{Context, ContextBuilder, State};

pub type GlContext = glow::web::Context;

pub struct Platform {}

impl Platform {
    pub fn new(builder: &ContextBuilder) -> Result<(Platform, GlContext, i32, i32)> {
        // TODO: This is disgusting
        let context = web_sys::window()
            .ok_or_else(|| TetraError::Platform("Could not get 'window' from browser".into()))?
            .document()
            .ok_or_else(|| TetraError::Platform("Could not get 'document' from browser".into()))?
            .get_element_by_id("canvas")
            .ok_or_else(|| TetraError::Platform("Could not find canvas element on page".into()))?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| TetraError::Platform("Element was not a canvas".into()))?
            .get_context("webgl2")
            .map_err(|_| TetraError::Platform("Could not get context from canvas".into()))?
            .expect("webgl2 is a valid context type")
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();

        Ok((
            Platform {},
            GlContext::from_webgl2_context(context),
            640,
            480,
        ))
    }
}

pub fn run_loop<S>(mut ctx: Context, mut state: S, frame: fn(&mut Context, &mut S))
where
    S: State + 'static,
{
    let callback = Rc::new(RefCell::new(None));
    let init = callback.clone();
    let refs = Rc::new(RefCell::new((ctx, state)));

    *init.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        let (ctx, state) = &mut *refs.borrow_mut();
        frame(ctx, state);

        if ctx.running {
            request_animation_frame(callback.borrow().as_ref().unwrap());
        }
    }) as Box<dyn FnMut()>));

    request_animation_frame(init.borrow().as_ref().unwrap());
}

pub fn handle_events(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn get_window_title(ctx: &Context) -> &str {
    ""
}

pub fn set_window_title<S>(ctx: &mut Context, title: S)
where
    S: AsRef<str>,
{

}

pub fn get_window_width(ctx: &Context) -> i32 {
    640
}

pub fn get_window_height(ctx: &Context) -> i32 {
    480
}

pub fn get_window_size(ctx: &Context) -> (i32, i32) {
    (640, 480)
}

pub fn set_window_size(ctx: &mut Context, width: i32, height: i32) {}

pub fn toggle_fullscreen(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn enable_fullscreen(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn disable_fullscreen(ctx: &mut Context) -> Result {
    Ok(())
}

pub fn is_fullscreen(ctx: &Context) -> bool {
    false
}

pub fn set_mouse_visible(ctx: &mut Context, mouse_visible: bool) {}

pub fn is_mouse_visible(ctx: &Context) -> bool {
    true
}

pub fn swap_buffers(ctx: &Context) {}

pub fn get_gamepad_name(ctx: &Context, platform_id: i32) -> String {
    String::new()
}

pub fn is_gamepad_vibration_supported(ctx: &Context, platform_id: i32) -> bool {
    false
}

pub fn set_gamepad_vibration(ctx: &mut Context, platform_id: i32, strength: f32) {}

pub fn start_gamepad_vibration(ctx: &mut Context, platform_id: i32, strength: f32, duration: u32) {}

pub fn stop_gamepad_vibration(ctx: &mut Context, platform_id: i32) {}

// TODO: Find a better way of stubbing the audio stuff out.

pub fn play_sound(
    ctx: &Context,
    sound: &Sound,
    playing: bool,
    repeating: bool,
    volume: f32,
    speed: f32,
) -> Result<SoundInstance> {
    let controls = Arc::new(RemoteControls {
        playing: AtomicBool::new(playing),
        repeating: AtomicBool::new(repeating),
        rewind: AtomicBool::new(false),
        volume: Mutex::new(volume),
        speed: Mutex::new(speed),
    });

    Ok(SoundInstance { controls })
}

pub fn set_master_volume(ctx: &mut Context, volume: f32) {}

pub fn get_master_volume(ctx: &mut Context) -> f32 {
    1.0
}

#[derive(Debug)]
pub struct DecoderError;

impl Display for DecoderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "dummy decoder error")
    }
}

impl Error for DecoderError {}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
