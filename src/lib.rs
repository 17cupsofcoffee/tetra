extern crate gl;
extern crate image;
pub extern crate nalgebra_glm as glm;
extern crate sdl2;

pub mod graphics;
pub mod util;

use glm::Mat4;
use graphics::opengl::GLDevice;
pub use sdl2::event::Event;
pub use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::{Duration, Instant};

pub trait State {
    fn event(&mut self, ctx: &mut Context, event: Event);
    fn update(&mut self, ctx: &mut Context);
    fn draw(&mut self, ctx: &mut Context, dt: f64);
}

pub struct Context {
    sdl: Sdl,
    pub window: Window,
    pub gl: GLDevice,
    running: bool,
    tick_rate: f64,
    pub(crate) projection_matrix: Mat4,
}

pub struct ContextBuilder<'a> {
    title: &'a str,
    width: u32,
    height: u32,
}

impl<'a> ContextBuilder<'a> {
    pub fn new() -> ContextBuilder<'a> {
        ContextBuilder {
            title: "Tetra",
            width: 1280,
            height: 720,
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

    pub fn build(self) -> Context {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video
            .window(self.title, self.width, self.height)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let gl = GLDevice::new(&video, &window);

        Context {
            sdl,
            window,
            gl,
            running: false,
            tick_rate: 1.0 / 60.0,
            projection_matrix: util::ortho(
                0.0,
                self.width as f32,
                self.height as f32,
                0.0,
                -1.0,
                1.0,
            ),
        }
    }
}

pub fn run<T: State>(ctx: &mut Context, state: &mut T) {
    let mut events = ctx.sdl.event_pump().unwrap();

    let mut last_time = Instant::now();
    let mut lag = Duration::from_secs(0);
    let tick_rate = util::f64_to_duration(ctx.tick_rate);

    ctx.running = true;

    while ctx.running {
        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => ctx.running = false, // TODO: Add a way to override this
                Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => ctx.running = false, // TODO: Make this an option,
                _ => {}
            }

            state.event(ctx, event);
        }

        let current_time = Instant::now();
        let elapsed = current_time - last_time;
        last_time = current_time;
        lag += elapsed;

        while lag >= tick_rate {
            state.update(ctx);
            lag -= tick_rate;
        }

        let dt = util::duration_to_f64(lag) / ctx.tick_rate;

        state.draw(ctx, dt);

        ctx.window.gl_swap_window();
    }
}

pub fn quit(ctx: &mut Context) {
    ctx.running = false;
}
