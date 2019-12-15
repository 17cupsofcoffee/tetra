//! Tetra is a simple 2D game framework written in Rust. It uses SDL2 for event handling and OpenGL 3.2+ for rendering.
//!
//! * [Website](https://tetra.seventeencups.net)
//! * [API Docs](https://docs.rs/tetra)
//! * [FAQ](https://tetra.seventeencups.net/FAQ)
//!
//! ## Features
//!
//! * XNA/MonoGame-inspired API
//! * Efficient 2D rendering, with draw call batching by default
//! * Easy input handling, via polling or events, with support for gamepads
//! * Deterministic game loop by default, à la [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)
//! * Common building blocks built-in, such as:
//!     * Font rendering
//!     * Cameras
//!     * Screen scaling
//!
//! ## Installation
//!
//! To add Tetra to your project, add the following line to your `Cargo.toml` file:
//!
//! ```toml
//! tetra = "0.3"
//! ```
//!
//! You will also need to install the SDL2 native libraries - full details are provided in the [documentation](https://tetra.seventeencups.net/installation).
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
//!     fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
//!         // Cornflower blue, as is tradition
//!         graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
//!         Ok(())
//!     }
//! }
//!
//! fn main() -> tetra::Result {
//!     ContextBuilder::new("Hello, world!", 1280, 720)
//!         .build()?
//!         .run(|_| Ok(GameState))
//! }
//! ```
//!
//! You can see this example in action by running `cargo run --example hello_world`.
//!
//! The full list of examples is available [here](https://github.com/17cupsofcoffee/tetra/tree/release/examples).
//!
//! ## Support/Feedback
//!
//! Tetra is fairly early in development, so you might run into bugs/flaky docs/general weirdness. Please feel free to open an issue/PR if you find something! You can also contact me via [Twitter](https://twitter.com/17cupsofcoffee), or find me lurking in the #games-and-graphics channel on the [Rust Community Discord](https://bit.ly/rust-community).

#![warn(missing_docs)]

pub mod audio;
pub mod error;
mod fs;
pub mod graphics;
pub mod input;
mod platform;
pub mod time;
pub mod window;
pub use vek as math;

use crate::audio::AudioContext;
pub use crate::error::{Result, TetraError};
use crate::graphics::opengl::GLDevice;
use crate::graphics::GraphicsContext;
use crate::input::{GamepadAxis, GamepadButton, GamepadStick, InputContext, Key, MouseButton};
use crate::math::Vec2;
use crate::platform::Platform;
use crate::time::{TimeContext, Timestep};

/// A trait representing a type that contains game state and provides logic for updating it
/// and drawing it to the screen. This is where you'll write your game logic!
///
/// The methods on `State` allow you to return a `Result`, either explicitly or via the `?`
/// operator. If an error is returned, the game will close and the error will be returned from
/// the `run` function that was used to start it.
#[allow(unused_variables)]
pub trait State {
    /// Called when it is time for the game to update.
    fn update(&mut self, ctx: &mut Context) -> Result {
        Ok(())
    }

    /// Called when it is time for the game to be drawn.
    fn draw(&mut self, ctx: &mut Context) -> Result {
        Ok(())
    }

    /// Called when a window or input event occurs.
    fn event(&mut self, ctx: &mut Context, event: Event) -> Result {
        Ok(())
    }
}

/// A struct containing all of the 'global' state within the framework.
pub struct Context {
    platform: Platform,
    gl: GLDevice,

    audio: AudioContext,
    graphics: GraphicsContext,
    input: InputContext,
    time: TimeContext,

    running: bool,
    quit_on_escape: bool,
}

impl Context {
    pub(crate) fn new(settings: &ContextBuilder) -> Result<Context> {
        // This needs to be initialized ASAP to avoid https://github.com/tomaka/rodio/issues/214
        let audio = AudioContext::new();

        let (platform, gl_context, window_width, window_height) = Platform::new(settings)?;
        let mut gl = GLDevice::new(gl_context)?;

        if settings.debug_info {
            println!("OpenGL Vendor: {}", gl.get_vendor());
            println!("OpenGL Renderer: {}", gl.get_renderer());
            println!("OpenGL Version: {}", gl.get_version());
            println!("GLSL Version: {}", gl.get_shading_language_version());
        }

        let graphics = GraphicsContext::new(&mut gl, window_width, window_height)?;
        let input = InputContext::new();
        let time = TimeContext::new(settings.timestep);

        Ok(Context {
            platform,
            gl,

            audio,
            graphics,
            input,
            time,

            running: false,
            quit_on_escape: settings.quit_on_escape,
        })
    }

    /// Runs the game.
    ///
    /// The `init` parameter takes a function or closure that creates a `State`
    /// implementation. A common pattern is to use method references to pass in
    /// your state's constructor directly - see the example below for how this
    /// works.
    ///
    /// # Errors
    ///
    /// If the `State` returns an error from `update` or `draw`, the game will stop
    /// running and this method will return the error.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tetra::{Context, ContextBuilder, State};
    ///
    /// struct GameState;
    ///
    /// impl GameState {
    ///     fn new(ctx: &mut Context) -> tetra::Result<GameState> {
    ///         Ok(GameState)
    ///     }
    /// }
    ///
    /// impl State for GameState { }
    ///
    /// fn main() -> tetra::Result {
    ///     // Because the signature of GameState::new is
    ///     // (&mut Context) -> tetra::Result<GameState>, you can pass it
    ///     // into run directly.
    ///     ContextBuilder::new("Hello, world!", 1280, 720)
    ///         .build()?
    ///         .run(GameState::new)
    /// }
    /// ```
    ///
    pub fn run<S, F>(&mut self, init: F) -> Result
    where
        S: State,
        F: FnOnce(&mut Context) -> Result<S>,
    {
        let state = &mut init(self)?;

        time::reset(self);

        self.running = true;

        while self.running {
            if let Err(e) = self.tick(state) {
                self.running = false;
                return Err(e);
            }
        }

        Ok(())
    }

    pub(crate) fn tick<S>(&mut self, state: &mut S) -> Result
    where
        S: State,
    {
        time::tick(self);

        platform::handle_events(self, state)?;

        match time::get_timestep(self) {
            Timestep::Fixed(_) => {
                while time::is_fixed_update_ready(self) {
                    state.update(self)?;
                    input::clear(self);
                }
            }
            Timestep::Variable => {
                state.update(self)?;
                input::clear(self);
            }
        }

        state.draw(self)?;

        graphics::present(self);

        std::thread::yield_now();

        Ok(())
    }
}

/// Events that can occur while the game is running.
///
/// The [`event` method on the `State` trait](trait.State.html#method.event) will recieve
/// events as they occur.
#[derive(Debug, Clone)]
pub enum Event {
    /// The game window was resized.
    Resized {
        /// The new width of the game window.
        width: i32,

        /// The new height of the game window.
        height: i32,
    },

    /// The game window was focused by the user.
    FocusGained,

    /// The game window was un-focused by the user.
    FocusLost,

    /// A key on the keyboard was pressed.
    KeyPressed {
        /// The key that was pressed.
        key: Key,
    },

    /// A key on the keyboard was released.
    KeyReleased {
        /// The key that was released.
        key: Key,
    },

    /// A button on the mouse was pressed.
    MouseButtonPressed {
        /// The button that was pressed.
        button: MouseButton,
    },

    /// A button on the mouse was released.
    MouseButtonReleased {
        /// The button that was released.
        button: MouseButton,
    },

    /// The mouse was moved.
    MouseMoved {
        /// The new position of the mouse, in window co-ordinates.
        position: Vec2<f32>,
    },

    /// A gamepad was connected to the system.
    GamepadAdded {
        /// The ID that was assigned to the gamepad.
        id: usize,
    },

    /// A gamepad was removed from the system.
    GamepadRemoved {
        /// The ID of the gamepad that was removed.
        id: usize,
    },

    /// A button on a gamepad was pressed.
    GamepadButtonPressed {
        /// The ID of the gamepad.
        id: usize,

        /// The button that was pressed.
        button: GamepadButton,
    },

    /// A button on a gamepad was released.
    GamepadButtonReleased {
        /// The ID of the gamepad.
        id: usize,

        /// The button that was released.
        button: GamepadButton,
    },

    /// An axis on a gamepad was moved.
    GamepadAxisMoved {
        /// The ID of the gamepad.
        id: usize,

        /// The axis that was moved.
        axis: GamepadAxis,

        /// The new position of the axis.
        position: f32,
    },

    /// A control stick on a gamepad was moved.
    GamepadStickMoved {
        /// The ID of the gamepad.
        id: usize,

        /// The stick that was moved.
        stick: GamepadStick,

        /// The new position of the stick.
        position: Vec2<f32>,
    },

    /// The user typed some text.
    TextInput {
        /// The text that was typed by the user.
        text: String,
    },

    /// This is here so that adding new error types will not be a breaking change.
    /// Can be removed once #[non_exhaustive] is stabilized.
    #[doc(hidden)]
    __Nonexhaustive,
}

/// Settings that can be configured when starting up a game.
#[derive(Debug, Clone)]
pub struct ContextBuilder {
    title: String,
    window_width: i32,
    window_height: i32,
    vsync: bool,
    timestep: Timestep,
    fullscreen: bool,
    maximized: bool,
    minimized: bool,
    resizable: bool,
    borderless: bool,
    show_mouse: bool,
    quit_on_escape: bool,
    debug_info: bool,
}

impl ContextBuilder {
    /// Create a new `ContextBuilder`, with a title and window size.
    pub fn new<S>(title: S, window_width: i32, window_height: i32) -> ContextBuilder
    where
        S: Into<String>,
    {
        ContextBuilder {
            title: title.into(),
            window_width,
            window_height,

            ..ContextBuilder::default()
        }
    }

    /// Sets the title of the window.
    ///
    /// Defaults to `"Tetra"`.
    pub fn title<S>(&mut self, title: S) -> &mut ContextBuilder
    where
        S: Into<String>,
    {
        self.title = title.into();
        self
    }

    /// Sets the size of the window.
    ///
    /// Defaults to `1280` by `720`.
    pub fn size(&mut self, width: i32, height: i32) -> &mut ContextBuilder {
        self.window_width = width;
        self.window_height = height;
        self
    }

    /// Enables or disables vsync.
    ///
    /// Defaults to `true`.
    pub fn vsync(&mut self, vsync: bool) -> &mut ContextBuilder {
        self.vsync = vsync;
        self
    }

    /// Sets the game's timestep.
    ///
    /// Defaults to `Timestep::Fixed(60.0)`.
    pub fn timestep(&mut self, timestep: Timestep) -> &mut ContextBuilder {
        self.timestep = timestep;
        self
    }

    /// Sets whether or not the window should start in fullscreen.
    ///
    /// Defaults to `false`.
    pub fn fullscreen(&mut self, fullscreen: bool) -> &mut ContextBuilder {
        self.fullscreen = fullscreen;
        self
    }

    /// Sets whether or not the window should start maximized.
    ///
    /// Defaults to `false`.
    pub fn maximized(&mut self, maximized: bool) -> &mut ContextBuilder {
        self.maximized = maximized;
        self
    }

    /// Sets whether or not the window should start minimized.
    ///
    /// Defaults to `false`.
    pub fn minimized(&mut self, minimized: bool) -> &mut ContextBuilder {
        self.minimized = minimized;
        self
    }

    /// Sets whether or not the window should be resizable.
    ///
    /// Defaults to `false`.
    pub fn resizable(&mut self, resizable: bool) -> &mut ContextBuilder {
        self.resizable = resizable;
        self
    }

    /// Sets whether or not the window should be borderless.
    ///
    /// Defaults to `false`.
    pub fn borderless(&mut self, borderless: bool) -> &mut ContextBuilder {
        self.borderless = borderless;
        self
    }

    /// Sets whether or not the mouse cursor should be visible when it is within the
    /// game window.
    ///
    /// Defaults to `false`.
    pub fn show_mouse(&mut self, show_mouse: bool) -> &mut ContextBuilder {
        self.show_mouse = show_mouse;
        self
    }

    /// Sets whether or not the game should close when the Escape key is pressed.
    ///
    /// Defaults to `false`.
    pub fn quit_on_escape(&mut self, quit_on_escape: bool) -> &mut ContextBuilder {
        self.quit_on_escape = quit_on_escape;
        self
    }

    /// Sets whether or not the game should print out debug info at startup.
    /// Please include this if you're submitting a bug report!
    pub fn debug_info(&mut self, debug_info: bool) -> &mut ContextBuilder {
        self.debug_info = debug_info;
        self
    }

    /// Builds the context.
    ///
    /// # Errors
    ///
    /// * `TetraError::PlatformError` will be returned if the context cannot be initialized.
    pub fn build(&self) -> Result<Context> {
        Context::new(self)
    }
}

impl Default for ContextBuilder {
    fn default() -> ContextBuilder {
        ContextBuilder {
            title: "Tetra".into(),
            window_width: 1280,
            window_height: 720,
            vsync: true,
            timestep: Timestep::Fixed(60.0),
            fullscreen: false,
            maximized: false,
            minimized: false,
            resizable: false,
            borderless: false,
            show_mouse: false,
            quit_on_escape: false,
            debug_info: false,
        }
    }
}
