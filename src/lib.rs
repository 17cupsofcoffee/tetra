//! Tetra is a simple 2D game framework written in Rust. It uses SDL2 for event handling and OpenGL 3.2+ for rendering.
//!
//! **Note that Tetra is still extremely early in development!** It may/will have bugs and missing features. That said, you're welcome to give it a go and let me know what you think :)
//!
//! ## Features
//!
//! * XNA/MonoGame-inspired API
//! * Efficient 2D rendering, with draw call batching by default
//! * Simple input handling
//! * Animations/spritesheets
//! * TTF font rendering
//! * Multiple screen scaling algorithms, including pixel-perfect variants (for those chunky retro pixels)
//! * Deterministic game loop, à la [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)
//!
//! ## Installation
//!
//! To add Tetra to your project, add the following line to your `Cargo.toml` file:
//!
//! ```toml
//! tetra = "0.2"
//! ```
//!
//! Tetra currently requires Rust 1.32 or higher.
//!
//! You will also need to install the SDL2 native libraries, as described [here](https://github.com/Rust-SDL2/rust-sdl2#user-content-requirements). The 'bundled' and 'static linking' features described can be activated using the `sdl2_bundled` and `sdl2_static_link` Cargo features in Tetra.
//!
//! ## Examples
//!
//! To get a simple window displayed on screen, the following code can be used:
//!
//! ```no_run
//! use tetra::graphics::{self, Color};
//! use tetra::{Context, ContextBuilder, State};
//!
//! struct GameState;
//!
//! impl State for GameState {
//!     fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
//!         // Cornflour blue, as is tradition
//!         graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
//!         Ok(())
//!     }
//! }
//!
//! fn main() -> tetra::Result {
//!     ContextBuilder::new("Hello, world!", 1280, 720)
//!         .build()?
//!         .run(&mut GameState)
//! }
//! ```
//!
//! You can see this example in action by running `cargo run --example hello_world`.
//!
//! The full list of examples is available [here](https://github.com/17cupsofcoffee/tetra/tree/master/examples).
//!
//! ## Support/Feedback
//!
//! As mentioned above, Tetra is fairly early in development, so there's likely to be bugs/flaky docs/general weirdness. Please feel free to leave an issue/PR if you find something!
//!
//! You can also contact me via [Twitter](https://twitter.com/17cupsofcoffee), or find me lurking in the #gamedev channel on the [Rust Community Discord](https://bit.ly/rust-community).

#![warn(missing_docs)]

pub mod audio;
pub mod error;
pub mod glm;
pub mod graphics;
pub mod input;
pub mod time;
pub mod window;

use sdl2::event::{Event, WindowEvent};
use sdl2::video::{FullscreenType, GLProfile, Window};
use sdl2::Sdl;

use crate::audio::AudioContext;
pub use crate::error::{Result, TetraError};
use crate::graphics::opengl::GLDevice;
use crate::graphics::GraphicsContext;
use crate::graphics::ScreenScaling;
use crate::input::InputContext;
use crate::time::TimeContext;

/// A trait representing a type that contains game state and provides logic for updating it
/// and drawing it to the screen. This is where you'll write your game logic!
///
/// The methods on `State` allow you to return a `Result`, either explicitly or via the `?`
/// operator. If an error is returned, the game will close and the error will be returned from
/// the `run` function that was used to start it.
#[allow(unused_variables)]
pub trait State {
    /// Called when it is time for the game to update, at the interval specified by the context's
    /// tick rate.
    ///
    /// The game will update at a fixed time step (defaulting to 60fps), and draw as fast as it is
    /// allowed to (depending on CPU/vsync settings). This allows for deterministic updates even at
    /// varying framerates, but may require you to do some interpolation in the `draw` method in
    /// order to make things look smooth.
    ///
    /// See [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/) for more info.
    fn update(&mut self, ctx: &mut Context) -> Result {
        Ok(())
    }

    /// Called when it is time for the game to be drawn.
    ///
    /// As drawing will not necessarily be in step with updating, the `dt` argument is provided -
    /// this will be a number between 0 and 1, specifying how far through the current tick you are.
    ///
    /// For example, if the player is meant to move 16 pixels per frame, and the current `dt` is 0.5,
    /// you should draw them 8 pixels along.
    fn draw(&mut self, ctx: &mut Context, dt: f64) -> Result {
        Ok(())
    }
}

/// A struct containing all of the 'global' state within the framework.
pub struct Context {
    sdl: Sdl,
    window: Window,
    gl: GLDevice,
    graphics: GraphicsContext,
    input: InputContext,
    audio: AudioContext,
    time: TimeContext,

    window_width: i32,
    window_height: i32,
    fullscreen: bool,
    running: bool,
    quit_on_escape: bool,
}

impl Context {
    /// Runs the game using the provided `State` implementation.
    ///
    /// # Errors
    ///
    /// If the `State` returns an error from `update` or `draw`, the game will stop
    /// running and this method will return the error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tetra::{Context, ContextBuilder, State};
    /// #
    /// struct GameState;
    ///
    /// impl State for GameState { }
    ///
    /// fn main() -> tetra::Result {
    ///    ContextBuilder::default().build()?.run(&mut GameState)
    /// }
    /// ```
    pub fn run<S>(&mut self, state: &mut S) -> Result
    where
        S: State,
    {
        self.running = true;
        self.window.show();
        time::reset(self);

        let mut events = self.sdl.event_pump().map_err(TetraError::Sdl)?;

        while self.running {
            time::tick(self);

            for event in events.poll_iter() {
                if let Err(e) = self
                    .handle_event(event)
                    .and_then(|event| input::handle_event(self, event))
                {
                    self.running = false;
                    return Err(e);
                }
            }

            while time::is_tick_ready(self) {
                if let Err(e) = state.update(self) {
                    self.running = false;
                    return Err(e);
                }

                input::cleanup_after_state_update(self);

                time::consume_tick(self);
            }

            if let Err(e) = state.draw(self, time::get_alpha(self)) {
                self.running = false;
                return Err(e);
            }

            graphics::present(self);

            std::thread::yield_now();
        }

        self.window.hide();

        Ok(())
    }

    /// Constructs an implementation of `State` using the given closure, and then runs it.
    ///
    /// This is mainly handy when chaining methods, as it allows you to call your `State` constructor
    /// without breaking the chain.
    ///
    /// # Errors
    ///
    /// If the `State` returns an error from `update` or `draw`, the game will stop
    /// running and this method will return the error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use tetra::graphics::Texture;
    /// # use tetra::{Context, ContextBuilder, State};
    /// #
    /// struct GameState {
    ///     texture: Texture,
    /// }
    ///
    /// impl GameState {
    ///     fn new(ctx: &mut Context) -> tetra::Result<GameState> {
    ///         Ok(GameState {
    ///             texture: Texture::new(ctx, "./examples/resources/player.png")?,
    ///         })
    ///     }
    /// }
    ///
    /// impl State for GameState { }
    ///
    /// fn main() -> tetra::Result {
    ///    ContextBuilder::default().build()?.run_with(GameState::new)
    /// }
    /// ```
    pub fn run_with<S, F>(&mut self, init: F) -> Result
    where
        S: State,
        F: FnOnce(&mut Context) -> Result<S>,
    {
        let state = &mut init(self)?;
        self.run(state)
    }

    fn handle_event(&mut self, event: Event) -> Result<Event> {
        match event {
            Event::Quit { .. } => self.running = false, // TODO: Add a way to override this
            Event::Window { win_event, .. } => {
                if let WindowEvent::SizeChanged(x, y) = win_event {
                    window::set_size_ex(self, x, y, true)
                }
            }
            _ => {}
        }

        Ok(event)
    }
}

/// Creates a new `Context` based on the provided options.
#[derive(Debug, Clone)]
pub struct ContextBuilder<'a> {
    title: &'a str,
    internal_width: i32,
    internal_height: i32,
    window_size: Option<(i32, i32)>,
    window_scale: Option<i32>,
    scaling: ScreenScaling,
    vsync: bool,
    tick_rate: f64,
    fullscreen: bool,
    maximized: bool,
    minimized: bool,
    resizable: bool,
    borderless: bool,
    show_mouse: bool,
    quit_on_escape: bool,
}

impl<'a> ContextBuilder<'a> {
    /// Creates a new ContextBuilder.
    pub fn new(title: &'a str, width: i32, height: i32) -> ContextBuilder<'a> {
        ContextBuilder {
            title,
            internal_width: width,
            internal_height: height,

            ..ContextBuilder::default()
        }
    }

    /// Sets the title of the window.
    ///
    /// Defaults to `"Tetra"`.
    pub fn title(&mut self, title: &'a str) -> &mut ContextBuilder<'a> {
        self.title = title;
        self
    }

    /// Sets the internal resolution of the screen.
    ///
    /// Defaults to `1280 x 720`.
    pub fn size(&mut self, width: i32, height: i32) -> &mut ContextBuilder<'a> {
        self.internal_width = width;
        self.internal_height = height;
        self
    }

    /// Sets the scaling mode for the game.
    ///
    /// Defaults to `ScreenScaling::ShowAllPixelPerfect`, which will maintain the screen's aspect ratio
    /// by letterboxing.
    pub fn scaling(&mut self, scaling: ScreenScaling) -> &mut ContextBuilder<'a> {
        self.scaling = scaling;
        self
    }

    /// Sets the size of the window.
    ///
    /// This only needs to be set if you want the internal resolution of the game
    /// to be different from the window size.
    ///
    /// This will take precedence over `window_scale`.
    pub fn window_size(&mut self, width: i32, height: i32) -> &mut ContextBuilder<'a> {
        self.window_size = Some((width, height));
        self
    }

    /// Sets the size of the window, as a multiplier of the internal screen size.
    ///
    /// This only needs to be set if you want the internal resolution of the game
    /// to be different from the window size.
    ///
    /// `window_size` will take precedence over this.
    pub fn window_scale(&mut self, scale: i32) -> &mut ContextBuilder<'a> {
        self.window_scale = Some(scale);
        self
    }

    /// Enables or disables vsync.
    ///
    /// Defaults to `true`.
    pub fn vsync(&mut self, vsync: bool) -> &mut ContextBuilder<'a> {
        self.vsync = vsync;
        self
    }

    /// Sets the game's update tick rate, in ticks per second.
    ///
    /// Defaults to `60.0`.
    pub fn tick_rate(&mut self, tick_rate: f64) -> &mut ContextBuilder<'a> {
        self.tick_rate = 1.0 / tick_rate;
        self
    }

    /// Sets whether or not the window should start in fullscreen.
    ///
    /// Defaults to `false`.
    pub fn fullscreen(&mut self, fullscreen: bool) -> &mut ContextBuilder<'a> {
        self.fullscreen = fullscreen;
        self
    }

    /// Sets whether or not the window should start maximized.
    ///
    /// Defaults to `false`.
    pub fn maximized(&mut self, maximized: bool) -> &mut ContextBuilder<'a> {
        self.maximized = maximized;
        self
    }

    /// Sets whether or not the window should start minimized.
    ///
    /// Defaults to `false`.
    pub fn minimized(&mut self, minimized: bool) -> &mut ContextBuilder<'a> {
        self.minimized = minimized;
        self
    }

    /// Sets whether or not the window should be resizable.
    ///
    /// Defaults to `false`.
    pub fn resizable(&mut self, resizable: bool) -> &mut ContextBuilder<'a> {
        self.resizable = resizable;
        self
    }

    /// Sets whether or not the window should be borderless.
    ///
    /// Defaults to `false`.
    pub fn borderless(&mut self, borderless: bool) -> &mut ContextBuilder<'a> {
        self.borderless = borderless;
        self
    }

    /// Sets whether or not the mouse cursor should be visible.
    ///
    /// Defaults to `false`.
    pub fn show_mouse(&mut self, show_mouse: bool) -> &mut ContextBuilder<'a> {
        self.show_mouse = show_mouse;
        self
    }

    /// Sets whether or not the game should close when the Escape key is pressed.
    ///
    /// Defaults to `false`.
    pub fn quit_on_escape(&mut self, quit_on_escape: bool) -> &mut ContextBuilder<'a> {
        self.quit_on_escape = quit_on_escape;
        self
    }

    /// Builds the context.
    ///
    /// # Errors
    ///
    /// If an error is encountered during initialization of the context, this method will
    /// return the error. This will usually be either `TetraError::Sdl` or `TetraError::OpenGl`.
    pub fn build(&self) -> Result<Context> {
        // This needs to be initialized ASAP to avoid https://github.com/tomaka/rodio/issues/214
        let audio = AudioContext::new();

        let sdl = sdl2::init().map_err(TetraError::Sdl)?;
        let video = sdl.video().map_err(TetraError::Sdl)?;

        let gl_attr = video.gl_attr();
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);
        gl_attr.set_red_size(8);
        gl_attr.set_green_size(8);
        gl_attr.set_blue_size(8);
        gl_attr.set_alpha_size(8);
        gl_attr.set_double_buffer(true);
        // TODO: Will need to add some more here if we start using the depth/stencil buffers

        let (mut window_width, mut window_height) = if let Some(size) = self.window_size {
            size
        } else if let Some(scale) = self.window_scale {
            (self.internal_width * scale, self.internal_height * scale)
        } else {
            (self.internal_width, self.internal_height)
        };

        let mut window_builder =
            video.window(self.title, window_width as u32, window_height as u32);

        window_builder.hidden().position_centered().opengl();

        if self.resizable {
            window_builder.resizable();
        }

        if self.borderless {
            window_builder.borderless();
        }

        sdl.mouse().show_cursor(self.show_mouse);

        let mut window = window_builder.build()?;

        // We wait until the window has been created to fiddle with this stuff as:
        // a) we don't want to blow away the window size settings
        // b) we don't know what monitor they're on until the window is created

        if self.maximized {
            window.maximize();
            let size = window.drawable_size();
            window_width = size.0 as i32;
            window_height = size.1 as i32;
        } else if self.minimized {
            window.minimize();
            let size = window.drawable_size();
            window_width = size.0 as i32;
            window_height = size.1 as i32;
        }

        if self.fullscreen {
            window
                .display_mode()
                .and_then(|m| {
                    window_width = m.w;
                    window_height = m.h;
                    window.set_fullscreen(FullscreenType::Desktop)
                })
                .map_err(TetraError::Sdl)?;
        }

        let mut gl = GLDevice::new(&video, &window, self.vsync)?;

        let graphics = GraphicsContext::new(
            &mut gl,
            window_width,
            window_height,
            self.internal_width,
            self.internal_height,
            self.scaling,
        )?;

        let input = InputContext::new(&sdl)?;
        let time = TimeContext::new(self.tick_rate);

        Ok(Context {
            sdl,
            window,
            gl,
            graphics,
            input,
            audio,
            time,

            window_width,
            window_height,
            fullscreen: self.fullscreen,
            running: false,
            quit_on_escape: self.quit_on_escape,
        })
    }
}

impl<'a> Default for ContextBuilder<'a> {
    fn default() -> ContextBuilder<'a> {
        ContextBuilder {
            title: "Tetra",
            internal_width: 1280,
            internal_height: 720,
            window_size: None,
            window_scale: None,
            scaling: ScreenScaling::ShowAllPixelPerfect,
            vsync: true,
            tick_rate: 1.0 / 60.0,
            fullscreen: false,
            maximized: false,
            minimized: false,
            resizable: false,
            borderless: false,
            show_mouse: false,
            quit_on_escape: false,
        }
    }
}
