extern crate gl;
extern crate image;
pub extern crate nalgebra_glm as glm;
extern crate sdl2;

pub mod graphics;
pub mod util;

use graphics::opengl::GLDevice;
pub use sdl2::event::Event;
pub use sdl2::keyboard::Keycode;
use sdl2::video::Window;
use sdl2::Sdl;
use std::time::{Duration, Instant};

const TICK_RATE: f64 = 1.0 / 60.0;

pub trait State {
    fn event(&mut self, app: &mut App, event: Event);
    fn update(&mut self, app: &mut App);
    fn draw(&mut self, app: &mut App, dt: f64);
}

pub struct App {
    sdl: Sdl,
    pub window: Window,
    pub gl: GLDevice,
    running: bool,
}

impl App {
    pub fn new(title: &str, width: u32, height: u32) -> App {
        let sdl = sdl2::init().unwrap();
        let video = sdl.video().unwrap();

        let window = video
            .window(title, width, height)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let gl = GLDevice::new(&video, &window);

        App {
            sdl,
            window,
            gl,
            running: false,
        }
    }

    pub fn run<T: State>(&mut self, mut state: T) {
        let mut events = self.sdl.event_pump().unwrap();

        let mut last_time = Instant::now();
        let mut lag = Duration::from_secs(0);
        let tick_rate = util::f64_to_duration(TICK_RATE);

        self.running = true;

        while self.running {
            for event in events.poll_iter() {
                match event {
                    Event::Quit { .. } => self.running = false, // TODO: Add a way to override this
                    Event::KeyDown {
                        keycode: Some(Keycode::Escape),
                        ..
                    } => self.running = false, // TODO: Make this an option,
                    _ => {}
                }

                state.event(self, event);
            }

            let current_time = Instant::now();
            let elapsed = current_time - last_time;
            last_time = current_time;
            lag += elapsed;

            while lag >= tick_rate {
                state.update(self);
                lag -= tick_rate;
            }

            let dt = util::duration_to_f64(lag) / TICK_RATE;

            state.draw(self, dt);

            self.window.gl_swap_window();
        }
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
