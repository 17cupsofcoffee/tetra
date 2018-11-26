extern crate gl;
extern crate image;
pub extern crate nalgebra_glm as glm;
extern crate sdl2;

pub mod error;
pub mod graphics;
pub mod input;
pub mod time;

use std::time::{Duration, Instant};

use glm::Vec2;
use sdl2::event::{Event, WindowEvent};
pub use sdl2::keyboard::Keycode as Key;
use sdl2::video::Window;
use sdl2::Sdl;

use error::{Result, TetraError};
use graphics::opengl::GLDevice;
use graphics::GraphicsContext;
use input::InputContext;

pub trait State {
    fn update(&mut self, ctx: &mut Context);
    fn draw(&mut self, ctx: &mut Context, dt: f64);
}

pub struct Context {
    sdl: Sdl,
    window: Window,
    gl: GLDevice,
    graphics: GraphicsContext,
    input: InputContext,

    running: bool,
    quit_on_escape: bool,
    tick_rate: Duration,
}

pub struct ContextBuilder<'a> {
    title: &'a str,
    width: i32,
    height: i32,
    scale: i32,
    vsync: bool,
    resizable: bool,
    tick_rate: f64,
    quit_on_escape: bool,
}

impl<'a> ContextBuilder<'a> {
    pub fn new() -> ContextBuilder<'a> {
        ContextBuilder {
            title: "Tetra",
            width: 1280,
            height: 720,
            scale: 1,
            vsync: true,
            resizable: false,
            tick_rate: 1.0 / 60.0,
            quit_on_escape: false,
        }
    }

    pub fn title(mut self, title: &'a str) -> ContextBuilder<'a> {
        self.title = title;
        self
    }

    pub fn size(mut self, width: i32, height: i32) -> ContextBuilder<'a> {
        self.width = width;
        self.height = height;
        self
    }

    pub fn scale(mut self, scale: i32) -> ContextBuilder<'a> {
        self.scale = scale;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> ContextBuilder<'a> {
        self.vsync = vsync;
        self
    }

    pub fn resizable(mut self, resizable: bool) -> ContextBuilder<'a> {
        self.resizable = resizable;
        self
    }

    pub fn tick_rate(mut self, tick_rate: f64) -> ContextBuilder<'a> {
        self.tick_rate = tick_rate;
        self
    }

    pub fn quit_on_escape(mut self, quit_on_escape: bool) -> ContextBuilder<'a> {
        self.quit_on_escape = quit_on_escape;
        self
    }

    pub fn build(self) -> Result<Context> {
        let sdl = sdl2::init().map_err(TetraError::Sdl)?;
        let video = sdl.video().map_err(TetraError::Sdl)?;

        let window_width = self.width * self.scale;
        let window_height = self.height * self.scale;

        let mut window_builder =
            video.window(self.title, window_width as u32, window_height as u32);

        window_builder.position_centered().opengl();

        if self.resizable {
            window_builder.resizable();
        }

        let window = window_builder
            .build()
            .map_err(|e| TetraError::Sdl(e.to_string()))?; // TODO: This could probably be cleaner

        let mut gl = GLDevice::new(&video, &window, self.vsync)?;
        let graphics = GraphicsContext::new(
            &mut gl,
            self.width,
            self.height,
            window_width,
            window_height,
        );
        let input = InputContext::new();

        Ok(Context {
            sdl,
            window,
            gl,
            graphics,
            input,

            running: false,
            quit_on_escape: self.quit_on_escape,
            tick_rate: time::f64_to_duration(self.tick_rate),
        })
    }
}

pub fn run<T: State>(ctx: &mut Context, state: &mut T) -> Result {
    let mut events = ctx.sdl.event_pump().map_err(TetraError::Sdl)?;

    let mut last_time = Instant::now();
    let mut lag = Duration::from_secs(0);

    ctx.running = true;

    while ctx.running {
        let current_time = Instant::now();
        let elapsed = current_time - last_time;
        last_time = current_time;
        lag += elapsed;

        for event in events.poll_iter() {
            handle_event(ctx, &event);
        }

        while lag >= ctx.tick_rate {
            state.update(ctx);
            ctx.input.previous_key_state = ctx.input.current_key_state;
            lag -= ctx.tick_rate;
        }

        let dt = time::duration_to_f64(lag) / time::duration_to_f64(ctx.tick_rate);

        state.draw(ctx, dt);

        graphics::present(ctx);

        std::thread::yield_now();
    }

    Ok(())
}

fn handle_event(ctx: &mut Context, event: &Event) {
    match event {
        Event::Quit { .. } => ctx.running = false, // TODO: Add a way to override this
        Event::KeyDown {
            keycode: Some(k), ..
        } => {
            if let Key::Escape = k {
                if ctx.quit_on_escape {
                    ctx.running = false;
                }
            }

            ctx.input.current_key_state[*k as usize] = true;
        }
        Event::KeyUp {
            keycode: Some(k), ..
        } => {
            // TODO: This can cause some inputs to be missed at low tick rates.
            // Could consider buffering input releases like Otter2D does?
            ctx.input.current_key_state[*k as usize] = false;
        }
        Event::MouseMotion { x, y, .. } => {
            ctx.input.mouse_position = Vec2::new(*x as f32, *y as f32)
        }
        Event::Window { win_event, .. } => if let WindowEvent::SizeChanged(x, y) = win_event {
            graphics::set_window_size(ctx, *x, *y)
        },
        _ => {}
    }
}

pub fn quit(ctx: &mut Context) {
    ctx.running = false;
}

pub fn set_tick_rate(ctx: &mut Context, tick_rate: f64) {
    ctx.tick_rate = time::f64_to_duration(tick_rate);
}
