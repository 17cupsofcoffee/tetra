use crate::graphics::{self, GraphicsContext};
use crate::input::{self, InputContext};
use crate::platform::{self, GraphicsDevice, Window};
use crate::time::{self, TimeContext, Timestep};
use crate::{Result, State};

#[cfg(feature = "audio")]
use crate::audio::AudioDevice;

/// A struct containing all of the 'global' state within the framework.
pub struct Context {
    pub(crate) window: Window,
    pub(crate) device: GraphicsDevice,
    #[cfg(feature = "audio")]
    pub(crate) audio: AudioDevice,
    pub(crate) graphics: GraphicsContext,
    pub(crate) input: InputContext,
    pub(crate) time: TimeContext,

    pub(crate) running: bool,
    pub(crate) quit_on_escape: bool,
}

impl Context {
    pub(crate) fn new(settings: &ContextBuilder) -> Result<Context> {
        // This needs to be initialized ASAP to avoid https://github.com/tomaka/rodio/issues/214
        #[cfg(feature = "audio")]
        let audio = AudioDevice::new();

        let (window, gl_context, window_width, window_height) = Window::new(settings)?;
        let mut device = GraphicsDevice::new(gl_context)?;

        if settings.debug_info {
            println!("OpenGL Vendor: {}", device.get_vendor());
            println!("OpenGL Renderer: {}", device.get_renderer());
            println!("OpenGL Version: {}", device.get_version());
            println!("GLSL Version: {}", device.get_shading_language_version());
        }

        let graphics = GraphicsContext::new(&mut device, window_width, window_height)?;
        let input = InputContext::new();
        let time = TimeContext::new(settings.timestep);

        Ok(Context {
            window,
            device,

            #[cfg(feature = "audio")]
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
        self.window.set_visible(true);

        let mut output = Ok(());

        while self.running {
            if let Err(e) = self.tick(state) {
                output = Err(e);
                self.running = false;
            }
        }

        self.window.set_visible(false);

        output
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

/// Settings that can be configured when starting up a game.
///
/// # Serde
///
/// Serialization and deserialization of this type (via [Serde](https://serde.rs/))
/// can be enabled via the `serde_support` feature.
///
/// Note that the available settings could change between releases of
/// Tetra (semver permitting). If you need a config file schema that will
/// be stable in the long term, consider making your own and then mapping
/// it to Tetra's API, rather than relying on `ContextBuilder` to not
/// change.
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct ContextBuilder {
    pub(crate) title: String,
    pub(crate) window_width: i32,
    pub(crate) window_height: i32,
    pub(crate) vsync: bool,
    pub(crate) timestep: Timestep,
    pub(crate) fullscreen: bool,
    pub(crate) maximized: bool,
    pub(crate) minimized: bool,
    pub(crate) resizable: bool,
    pub(crate) borderless: bool,
    pub(crate) show_mouse: bool,
    pub(crate) grab_mouse: bool,
    pub(crate) relative_mouse_mode: bool,
    pub(crate) quit_on_escape: bool,
    pub(crate) debug_info: bool,
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

    /// Sets whether or not the mouse cursor should be grabbed by the game window
    /// at startup.
    ///
    /// Defaults to `false`.
    pub fn grab_mouse(&mut self, grab_mouse: bool) -> &mut ContextBuilder {
        self.grab_mouse = grab_mouse;
        self
    }

    /// Sets whether or not relative mouse mode should be enabled.
    ///
    /// While the mouse is in relative mode, the cursor is hidden and can move beyond the
    /// bounds of the window. The [`delta` field of `Event::MouseMoved`](./enum.Event.html#variant.MouseMoved.field.delta)
    /// can then be used to track the cursor's changes in position. This is useful when
    /// implementing control schemes that require the mouse to be able to move infinitely
    /// in any direction (for example, FPS-style movement).
    ///
    /// While this mode is enabled, the absolute position of the mouse may not be updated -
    /// as such, you should not rely on it.
    ///
    /// Defaults to `false`.
    pub fn relative_mouse_mode(&mut self, relative_mouse_mode: bool) -> &mut ContextBuilder {
        self.relative_mouse_mode = relative_mouse_mode;
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
            grab_mouse: false,
            relative_mouse_mode: false,
            quit_on_escape: false,
            debug_info: false,
        }
    }
}
