//! Tetra is a simple 2D game framework written in Rust. It uses SDL2 for event handling and OpenGL 3.2+ for rendering.
//!
//! **Note that Tetra is still extremely early in development!** It may/will have bugs and missing features (the big ones currently being sound and gamepad support). That said, you're welcome to give it a go and let me know what you think :)
//!
//! ## Features
//!
//! * XNA/MonoGame-inspired API
//! * Efficient 2D rendering, with draw call batching by default
//! * Animations/spritesheets
//! * Pixel-perfect screen scaling
//! * Deterministic game loop, à la [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/).
//!
//! ## Installation
//!
//! To add Tetra to your project, add the following line to your `Cargo.toml` file:
//!
//! ```toml
//! tetra = "0.1"
//! ```
//!
//! You will also need to install the SDL2 native libraries, as described [here](https://github.com/Rust-SDL2/rust-sdl2#user-content-requirements).
//!
//! ## Examples
//!
//! To get a simple window displayed on screen, the following code can be used:
//!
//! ```no_run
//! extern crate tetra;
//!
//! use tetra::error::Result;
//! use tetra::graphics::{self, Color};
//! use tetra::{Context, ContextBuilder, State};
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn update(&mut self, _ctx: &mut Context) {}
//!
//!     fn draw(&mut self, ctx: &mut Context, _dt: f64) {
//!         // Cornflour blue, as is tradition
//!         graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
//!     }
//! }
//!
//! fn main() -> Result {
//!     let ctx = &mut ContextBuilder::new()
//!         .title("Hello, world!")
//!         .quit_on_escape(true)
//!         .build()?;
//!
//!     let state = &mut GameState;
//!
//!     tetra::run(ctx, state)
//! }
//! ```
//!
//! You can see this example in action by running `cargo run --example hello_world`.
//!
//! The full list of examples available are:
//!
//! * [`hello_world`](examples/hello_world.rs) - Opens a window and clears it with a solid color.
//! * [`texture`](examples/texture.rs) - Loads and displays a texture.
//! * [`keyboard`](examples/keyboard.rs) - Moves a texture around based on keyboard input.
//! * [`mouse`](examples/mouse.rs) - Moves a texture around based on mouse input.
//! * [`tetras`](examples/tetras.rs) - A full example game (which is entirely legally distinct from a certain other block-based puzzle game *cough*).
//!
//! ## Support/Feedback
//!
//! As mentioned above, Tetra is fairly early in development, so there's likely to be bugs/flaky docs/general weirdness. Please feel free to leave an issue/PR if you find something!
//!
//! You can also contact me via [Twitter](https://twitter.com/17cupsofcoffee), or find me lurking in the #gamedev channel on the [Rust Community Discord](https://bit.ly/rust-community).

#![deny(missing_docs)]

extern crate fnv;
extern crate gl;
extern crate image;
extern crate sdl2;

pub extern crate nalgebra_glm as glm;
pub mod error;
pub mod graphics;
pub mod input;
pub mod time;

use std::time::{Duration, Instant};

use glm::Vec2;
use sdl2::event::{Event, WindowEvent};
use sdl2::video::Window;
use sdl2::Sdl;

use error::{Result, TetraError};
use graphics::opengl::GLDevice;
use graphics::GraphicsContext;
use input::{InputContext, Key};

/// A trait representing a type that contains game state and provides logic for updating it
/// and drawing it to the screen. This is where you'll write your game logic!
///
/// The game will update at a fixed time step (defaulting to 60fps), and draw as fast as it is
/// allowed to (depending on CPU/vsync settings). This allows for deterministic updates even at
/// varying framerates, but may require you to do some interpolation in order to make things look
/// smooth.
///
/// See [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/) for more info.
pub trait State {
    /// Called when it is time for the game to update, at the interval specified by the context's
    /// tick rate.
    fn update(&mut self, ctx: &mut Context);

    /// Called when it is time for the game to be drawn.
    ///
    /// As drawing will not necessarily be in step with updating, the `dt` argument is provided -
    /// this will be a number between 0 and 1, specifying how far through the current tick you are.
    ///
    /// For example, if the player is meant to move 16 pixels per frame, and the current `dt` is 0.5,
    /// you should draw them 8 pixels along.
    fn draw(&mut self, ctx: &mut Context, dt: f64);
}

/// A struct containing all of the 'global' state within the framework.
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

/// Creates a new `Context` based on the provided options.
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
    /// Creates a new ContextBuilder, with the default settings.
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

    /// Sets the title of the window.
    pub fn title(mut self, title: &'a str) -> ContextBuilder<'a> {
        self.title = title;
        self
    }

    /// Sets the internal size of the screen.
    pub fn size(mut self, width: i32, height: i32) -> ContextBuilder<'a> {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the initial scale of the window, relative to the internal screen size.
    pub fn scale(mut self, scale: i32) -> ContextBuilder<'a> {
        self.scale = scale;
        self
    }

    /// Enables or disables vsync.
    pub fn vsync(mut self, vsync: bool) -> ContextBuilder<'a> {
        self.vsync = vsync;
        self
    }

    /// Sets whether or not the window should be resizable.
    pub fn resizable(mut self, resizable: bool) -> ContextBuilder<'a> {
        self.resizable = resizable;
        self
    }

    /// Sets the game's update tick rate.
    pub fn tick_rate(mut self, tick_rate: f64) -> ContextBuilder<'a> {
        self.tick_rate = tick_rate;
        self
    }

    /// Sets whether or not the game should close when the Escape key is pressed.
    pub fn quit_on_escape(mut self, quit_on_escape: bool) -> ContextBuilder<'a> {
        self.quit_on_escape = quit_on_escape;
        self
    }

    /// Builds the context.
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

        let mut window = window_builder
            .build()
            .map_err(|e| TetraError::Sdl(e.to_string()))?; // TODO: This could probably be cleaner

        window
            .set_minimum_size(self.width as u32, self.height as u32)
            .map_err(|e| TetraError::Sdl(e.to_string()))?;

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

/// Runs the game.
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
            ctx.input.previous_key_state = ctx.input.current_key_state.clone();
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

            ctx.input.current_key_state.insert(*k);
        }
        Event::KeyUp {
            keycode: Some(k), ..
        } => {
            // TODO: This can cause some inputs to be missed at low tick rates.
            // Could consider buffering input releases like Otter2D does?
            ctx.input.current_key_state.remove(k);
        }
        Event::MouseButtonDown { mouse_btn, .. } => {
            ctx.input.current_mouse_state.insert(*mouse_btn);
        }
        Event::MouseButtonUp { mouse_btn, .. } => {
            ctx.input.current_mouse_state.remove(mouse_btn);
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

/// Quits the game, if it is currently running.
///
/// Note that currently, quitting the game does not take effect until the end of the current
/// cycle of the game loop. This will probably change later.
pub fn quit(ctx: &mut Context) {
    ctx.running = false;
}

/// Sets the update tick rate of the application, in ticks per second.
pub fn set_tick_rate(ctx: &mut Context, tick_rate: f64) {
    ctx.tick_rate = time::f64_to_duration(tick_rate);
}
