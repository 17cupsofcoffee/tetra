use std::cell::RefCell;
use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::rc::Rc;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use wasm_bindgen::convert::FromWasmAbi;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{console, EventTarget, HtmlCanvasElement, KeyboardEvent, MouseEvent};

use crate::audio::{RemoteControls, Sound, SoundInstance};
use crate::error::{Result, TetraError};
use crate::input::{self, Key, MouseButton};
use crate::math::Vec2;
use crate::{Context, Game, State};

const HIDE_CURSOR_CLASS: &str = "tetra-hide-cursor";
const FULLSCREEN_CLASS: &str = "tetra-fullscreen";

const STYLES: &str = r#"
    <style>
        .tetra-hide-cursor {
            cursor: none;
        }

        .tetra-fullscreen {
            position: fixed;
            top: 0;
            left: 0;
            width: 100vw;
            height: 100vh;
        }
    </style>
"#;

pub type GlContext = glow::web::Context;

enum Event {
    KeyDown(Key),
    KeyUp(Key),
    MouseDown(MouseButton),
    MouseUp(MouseButton),
    MouseMove(Vec2),
}

pub struct Platform {
    canvas: HtmlCanvasElement,

    event_queue: Rc<RefCell<VecDeque<Event>>>,

    _keydown_closure: Closure<dyn FnMut(KeyboardEvent)>,
    _keyup_closure: Closure<dyn FnMut(KeyboardEvent)>,
    _mousedown_closure: Closure<dyn FnMut(MouseEvent)>,
    _mouseup_closure: Closure<dyn FnMut(MouseEvent)>,
    _mousemove_closure: Closure<dyn FnMut(MouseEvent)>,
}

impl Platform {
    pub fn new(builder: &Game) -> Result<(Platform, GlContext, i32, i32)> {
        // TODO: This is disgusting
        let document = web_sys::window()
            .ok_or_else(|| TetraError::PlatformError("Could not get 'window' from browser".into()))?
            .document()
            .ok_or_else(|| {
                TetraError::PlatformError("Could not get 'document' from browser".into())
            })?;

        let canvas = document
            .get_element_by_id(&builder.canvas_id)
            .ok_or_else(|| {
                TetraError::PlatformError("Could not find canvas element on page".into())
            })?
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .map_err(|_| TetraError::PlatformError("Element was not a canvas".into()))?;

        canvas.set_width(builder.window_width as u32);
        canvas.set_height(builder.window_height as u32);

        canvas
            .insert_adjacent_html("afterend", STYLES)
            .map_err(|_| TetraError::PlatformError("Could not inject styles".into()))?;

        let class_list = canvas.class_list();

        if !builder.show_mouse {
            class_list.add_1(HIDE_CURSOR_CLASS).map_err(|_| {
                TetraError::PlatformError("Failed to modify canvas CSS classes".into())
            })?;
        }

        if builder.fullscreen {
            canvas.class_list().add_1(FULLSCREEN_CLASS).map_err(|_| {
                TetraError::PlatformError("Failed to modify canvas CSS classes".into())
            })?;
        }

        let context = canvas
            .get_context("webgl2")
            .map_err(|_| TetraError::PlatformError("Could not get context from canvas".into()))?
            .expect("webgl2 is a valid context type")
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .expect("returned value should be a webgl2 context");

        let event_queue = Rc::new(RefCell::new(VecDeque::new()));

        let event_queue_handle = Rc::clone(&event_queue);

        let _keydown_closure = event(&document, "keydown", move |event: KeyboardEvent| {
            if let Some(key) = into_key(event) {
                event_queue_handle
                    .borrow_mut()
                    .push_back(Event::KeyDown(key));
            }
        })?;

        let event_queue_handle = Rc::clone(&event_queue);

        let _keyup_closure = event(&document, "keyup", move |event: KeyboardEvent| {
            if let Some(key) = into_key(event) {
                event_queue_handle.borrow_mut().push_back(Event::KeyUp(key));
            }
        })?;

        let event_queue_handle = Rc::clone(&event_queue);

        let _mousedown_closure = event(&canvas, "mousedown", move |event: MouseEvent| {
            if let Some(btn) = into_mouse_button(event) {
                event_queue_handle
                    .borrow_mut()
                    .push_back(Event::MouseDown(btn));
            }
        })?;

        let event_queue_handle = Rc::clone(&event_queue);

        let _mouseup_closure = event(&canvas, "mouseup", move |event: MouseEvent| {
            if let Some(btn) = into_mouse_button(event) {
                event_queue_handle
                    .borrow_mut()
                    .push_back(Event::MouseUp(btn));
            }
        })?;

        let event_queue_handle = Rc::clone(&event_queue);

        let _mousemove_closure = event(&canvas, "mousemove", move |event: MouseEvent| {
            event_queue_handle
                .borrow_mut()
                .push_back(Event::MouseMove(Vec2::new(
                    event.offset_x() as f32,
                    event.offset_y() as f32,
                )));
        })?;

        Ok((
            Platform {
                canvas,

                event_queue,

                _keydown_closure,
                _keyup_closure,
                _mousedown_closure,
                _mouseup_closure,
                _mousemove_closure,
            },
            GlContext::from_webgl2_context(context),
            builder.window_width,
            builder.window_height,
        ))
    }
}

pub fn run_loop<S>(ctx: Context, state: S, frame: fn(&mut Context, &mut S))
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
    while let Some(event) = {
        let mut x = ctx.platform.event_queue.borrow_mut();
        x.pop_front()
    } {
        match event {
            Event::KeyDown(key) => input::set_key_down(ctx, key),
            Event::KeyUp(key) => input::set_key_up(ctx, key),
            Event::MouseDown(btn) => input::set_mouse_button_down(ctx, btn),
            Event::MouseUp(btn) => input::set_mouse_button_up(ctx, btn),
            Event::MouseMove(pos) => input::set_mouse_position(ctx, pos),
        }
    }

    Ok(())
}

pub fn log_info(info: &str) {
    console::info_1(&info.into());
}

pub fn log_error(error: TetraError) {
    console::error_1(&format!("Error: {}", error).into());
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
    ctx.platform.canvas.width() as i32
}

pub fn get_window_height(ctx: &Context) -> i32 {
    ctx.platform.canvas.height() as i32
}

pub fn get_window_size(ctx: &Context) -> (i32, i32) {
    (
        ctx.platform.canvas.width() as i32,
        ctx.platform.canvas.height() as i32,
    )
}

pub fn set_window_size(ctx: &mut Context, width: i32, height: i32) {
    ctx.platform.canvas.set_width(width as u32);
    ctx.platform.canvas.set_height(height as u32);
}

pub fn set_vsync(ctx: &mut Context, vsync: bool) -> Result {
    if vsync {
        Ok(())
    } else {
        Err(TetraError::FailedToChangeDisplayMode(
            "VSync cannot be disabled on web platforms.".into(),
        ))
    }
}

pub fn is_vsync_enabled(ctx: &Context) -> bool {
    true
}

pub fn set_fullscreen(ctx: &mut Context, fullscreen: bool) -> Result {
    let class_list = ctx.platform.canvas.class_list();

    if fullscreen {
        class_list.add_1(FULLSCREEN_CLASS)
    } else {
        class_list.remove_1(FULLSCREEN_CLASS)
    }
    .map_err(|_| {
        TetraError::FailedToChangeDisplayMode("Failed to modify canvas CSS classes".into())
    })
}

pub fn is_fullscreen(ctx: &Context) -> bool {
    ctx.platform.canvas.class_list().contains(FULLSCREEN_CLASS)
}

pub fn set_mouse_visible(ctx: &mut Context, mouse_visible: bool) -> Result {
    let class_list = ctx.platform.canvas.class_list();

    if mouse_visible {
        class_list.remove_1(HIDE_CURSOR_CLASS)
    } else {
        class_list.add_1(HIDE_CURSOR_CLASS)
    }
    .map_err(|_| TetraError::PlatformError("Failed to modify canvas CSS classes".into()))
}

pub fn is_mouse_visible(ctx: &Context) -> bool {
    !ctx.platform.canvas.class_list().contains(HIDE_CURSOR_CLASS)
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

fn event<F, T>(target: &EventTarget, event_name: &str, callback: F) -> Result<Closure<dyn FnMut(T)>>
where
    F: FnMut(T) + 'static,
    T: FromWasmAbi + 'static,
{
    let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(T)>);

    target
        .add_event_listener_with_callback(event_name, callback.as_ref().unchecked_ref())
        .map_err(|_| TetraError::PlatformError("Failed to create event listener".into()))?;

    Ok(callback)
}

fn into_key(event: KeyboardEvent) -> Option<Key> {
    let location = event.location();

    match (event.key().as_ref(), event.location()) {
        ("a", _) | ("A", _) => Some(Key::A),
        ("b", _) | ("B", _) => Some(Key::B),
        ("c", _) | ("C", _) => Some(Key::C),
        ("d", _) | ("D", _) => Some(Key::D),
        ("e", _) | ("E", _) => Some(Key::E),
        ("f", _) | ("F", _) => Some(Key::F),
        ("g", _) | ("G", _) => Some(Key::G),
        ("h", _) | ("H", _) => Some(Key::H),
        ("i", _) | ("I", _) => Some(Key::I),
        ("j", _) | ("J", _) => Some(Key::J),
        ("k", _) | ("K", _) => Some(Key::K),
        ("l", _) | ("L", _) => Some(Key::L),
        ("m", _) | ("M", _) => Some(Key::M),
        ("n", _) | ("N", _) => Some(Key::N),
        ("o", _) | ("O", _) => Some(Key::O),
        ("p", _) | ("P", _) => Some(Key::P),
        ("q", _) | ("Q", _) => Some(Key::Q),
        ("r", _) | ("R", _) => Some(Key::R),
        ("s", _) | ("S", _) => Some(Key::S),
        ("t", _) | ("T", _) => Some(Key::T),
        ("u", _) | ("U", _) => Some(Key::U),
        ("v", _) | ("V", _) => Some(Key::V),
        ("w", _) | ("W", _) => Some(Key::W),
        ("x", _) | ("X", _) => Some(Key::X),
        ("y", _) | ("Y", _) => Some(Key::Y),
        ("z", _) | ("Z", _) => Some(Key::Z),

        ("0", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num0),
        ("1", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num1),
        ("2", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num2),
        ("3", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num3),
        ("4", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num4),
        ("5", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num5),
        ("6", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num6),
        ("7", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num7),
        ("8", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num8),
        ("9", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Num9),

        ("F1", _) => Some(Key::F1),
        ("F2", _) => Some(Key::F2),
        ("F3", _) => Some(Key::F3),
        ("F4", _) => Some(Key::F4),
        ("F5", _) => Some(Key::F5),
        ("F6", _) => Some(Key::F6),
        ("F7", _) => Some(Key::F7),
        ("F8", _) => Some(Key::F8),
        ("F9", _) => Some(Key::F9),
        ("F10", _) => Some(Key::F10),
        ("F11", _) => Some(Key::F11),
        ("F12", _) => Some(Key::F12),
        ("F13", _) => Some(Key::F13),
        ("F14", _) => Some(Key::F14),
        ("F15", _) => Some(Key::F15),
        ("F16", _) => Some(Key::F16),
        ("F17", _) => Some(Key::F17),
        ("F18", _) => Some(Key::F18),
        ("F19", _) => Some(Key::F19),
        ("F20", _) => Some(Key::F20),
        ("F21", _) => Some(Key::F21),
        ("F22", _) => Some(Key::F22),
        ("F23", _) => Some(Key::F23),
        ("F24", _) => Some(Key::F24),

        ("NumLock", _) => Some(Key::NumLock),
        ("0", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad1),
        ("1", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad2),
        ("2", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad3),
        ("3", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad4),
        ("4", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad5),
        ("5", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad6),
        ("6", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad7),
        ("7", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad8),
        ("8", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad9),
        ("9", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPad0),
        ("+", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadPlus),
        ("-", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadMinus),
        ("*", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadMultiply),
        ("/", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadDivide),
        ("Enter", KeyboardEvent::DOM_KEY_LOCATION_NUMPAD) => Some(Key::NumPadEnter),

        ("Control", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::LeftCtrl),
        ("Shift", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::LeftShift),
        ("Alt", KeyboardEvent::DOM_KEY_LOCATION_LEFT) => Some(Key::LeftAlt),
        ("Control", KeyboardEvent::DOM_KEY_LOCATION_RIGHT) => Some(Key::RightCtrl),
        ("Shift", KeyboardEvent::DOM_KEY_LOCATION_RIGHT) => Some(Key::RightShift),
        ("Alt", KeyboardEvent::DOM_KEY_LOCATION_RIGHT) => Some(Key::RightAlt),

        ("ArrowUp", _) => Some(Key::Up),
        ("ArrowDown", _) => Some(Key::Down),
        ("ArrowLeft", _) => Some(Key::Left),
        ("ArrowRight", _) => Some(Key::Right),

        ("&", _) => Some(Key::Ampersand),
        ("*", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Asterisk),
        ("@", _) => Some(Key::At),
        ("`", _) => Some(Key::Backquote),
        ("\\", _) => Some(Key::Backslash),
        ("Backspace", _) => Some(Key::Backspace),
        ("CapsLock", _) => Some(Key::CapsLock),
        ("^", _) => Some(Key::Caret),
        (":", _) => Some(Key::Colon),
        (",", _) => Some(Key::Comma),
        ("Delete", _) => Some(Key::Delete),
        ("$", _) => Some(Key::Dollar),
        ("\"", _) => Some(Key::DoubleQuote),
        ("End", _) => Some(Key::End),
        ("Enter", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Enter),
        ("=", _) => Some(Key::Equals),
        ("Escape", _) => Some(Key::Escape),
        ("!", _) => Some(Key::Exclaim),
        (">", _) => Some(Key::GreaterThan),
        ("#", _) => Some(Key::Hash),
        ("Home", _) => Some(Key::Home),
        ("Insert", _) => Some(Key::Insert),
        ("{", _) => Some(Key::LeftBracket),
        ("(", _) => Some(Key::LeftParen),
        ("<", _) => Some(Key::LessThan),
        ("-", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Minus),
        ("PageDown", _) => Some(Key::PageDown),
        ("PageUp", _) => Some(Key::PageUp),
        ("Pause", _) => Some(Key::Pause),
        ("%", _) => Some(Key::Percent),
        (".", _) => Some(Key::Period),
        ("+", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Plus),
        ("PrintScreen", _) => Some(Key::PrintScreen),
        ("?", _) => Some(Key::Question),
        ("'", _) => Some(Key::Quote),
        ("}", _) => Some(Key::RightBracket),
        (")", _) => Some(Key::RightParen),
        ("ScrollLock", _) => Some(Key::ScrollLock),
        (";", _) => Some(Key::Semicolon),
        ("/", KeyboardEvent::DOM_KEY_LOCATION_STANDARD) => Some(Key::Slash),
        (" ", _) => Some(Key::Space),
        ("Tab", _) => Some(Key::Tab),
        ("_", _) => Some(Key::Underscore),

        _ => None,
    }
}

fn into_mouse_button(event: MouseEvent) -> Option<MouseButton> {
    match event.button() {
        0 => Some(MouseButton::Left),
        1 => Some(MouseButton::Middle),
        2 => Some(MouseButton::Right),
        3 => Some(MouseButton::X1),
        4 => Some(MouseButton::X2),
        _ => None,
    }
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
