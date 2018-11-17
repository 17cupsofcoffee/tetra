extern crate gl;
extern crate image;
pub extern crate nalgebra_glm as glm;
extern crate sdl2;

pub mod error;
pub mod graphics;
pub mod input;
pub mod util;

use error::{Result, TetraError};
use glm::Mat4;
use graphics::opengl::GLDevice;
use graphics::RenderState;
use sdl2::event::Event;
pub use sdl2::keyboard::Keycode as Key;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::{Duration, Instant};

pub trait State {
    fn update(&mut self, ctx: &mut Context);
    fn draw(&mut self, ctx: &mut Context, dt: f64);
}

pub struct Context {
    sdl: Sdl,
    pub window: Window,
    pub gl: GLDevice,
    pub render_state: RenderState,
    running: bool,
    quit_on_escape: bool,
    tick_rate: f64,
    pub(crate) projection_matrix: Mat4,
    pub(crate) current_key_state: [bool; 322],
    pub(crate) previous_key_state: [bool; 322],
}

pub struct ContextBuilder<'a> {
    title: &'a str,
    width: u32,
    height: u32,
    vsync: bool,
    quit_on_escape: bool,
}

impl<'a> ContextBuilder<'a> {
    pub fn new() -> ContextBuilder<'a> {
        ContextBuilder {
            title: "Tetra",
            width: 1280,
            height: 720,
            vsync: true,
            quit_on_escape: false,
        }
    }

    pub fn title(mut self, title: &'a str) -> ContextBuilder<'a> {
        self.title = title;
        self
    }

    pub fn window_size(mut self, width: u32, height: u32) -> ContextBuilder<'a> {
        self.width = width;
        self.height = height;
        self
    }

    pub fn vsync(mut self, vsync: bool) -> ContextBuilder<'a> {
        self.vsync = vsync;
        self
    }

    pub fn quit_on_escape(mut self, quit_on_escape: bool) -> ContextBuilder<'a> {
        self.quit_on_escape = quit_on_escape;
        self
    }

    pub fn build(self) -> Result<Context> {
        let sdl = sdl2::init().map_err(TetraError::Sdl)?;
        let video = sdl.video().map_err(TetraError::Sdl)?;

        let window = video
            .window(self.title, self.width, self.height)
            .position_centered()
            .opengl()
            .build()
            .map_err(|e| TetraError::Sdl(e.to_string()))?; // TODO: This could probably be cleaner

        let mut gl = GLDevice::new(&video, &window, self.vsync)?;
        let render_state = RenderState::new(&mut gl);

        Ok(Context {
            sdl,
            window,
            gl,
            render_state,
            running: false,
            quit_on_escape: self.quit_on_escape,
            tick_rate: 1.0 / 60.0,
            projection_matrix: util::ortho(
                0.0,
                self.width as f32,
                self.height as f32,
                0.0,
                -1.0,
                1.0,
            ),
            current_key_state: [false; 322],
            previous_key_state: [false; 322],
        })
    }
}

pub fn run<T: State>(ctx: &mut Context, state: &mut T) -> Result {
    let mut events = ctx.sdl.event_pump().map_err(TetraError::Sdl)?;

    let mut last_time = Instant::now();
    let mut lag = Duration::from_secs(0);
    let tick_rate = util::f64_to_duration(ctx.tick_rate);

    ctx.running = true;

    while ctx.running {
        let current_time = Instant::now();
        let elapsed = current_time - last_time;
        last_time = current_time;
        lag += elapsed;

        ctx.previous_key_state = ctx.current_key_state;

        for event in events.poll_iter() {
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

                    ctx.current_key_state[k as usize] = true;
                }
                Event::KeyUp {
                    keycode: Some(k), ..
                } => {
                    ctx.current_key_state[k as usize] = false;
                }
                _ => {}
            }
        }

        while lag >= tick_rate {
            state.update(ctx);
            lag -= tick_rate;
        }

        let dt = util::duration_to_f64(lag) / ctx.tick_rate;

        state.draw(ctx, dt);

        graphics::flush(ctx);

        ctx.window.gl_swap_window();

        std::thread::yield_now();
    }

    Ok(())
}

pub fn quit(ctx: &mut Context) {
    ctx.running = false;
}
