use std::result;
use std::thread;
use std::time::{Duration, Instant};

use crate::graphics::{self, GraphicsContext};
use crate::input::{self, InputContext};
use crate::platform::{self, GraphicsDevice, Window};
use crate::time::{self, TimeContext, Timestep};
use crate::{Result, State, TetraError};

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
            let device_info = device.get_info();

            println!("OpenGL Vendor: {}", device_info.vendor);
            println!("OpenGL Renderer: {}", device_info.renderer);
            println!("OpenGL Version: {}", device_info.opengl_version);
            println!("GLSL Version: {}", device_info.glsl_version);
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
    /// The `init` parameter takes a function or closure that creates a
    /// `State` implementation. A common pattern is to use method references
    /// to pass in your state's constructor directly - see the example below
    /// for how this works.
    ///
    /// The error type returned by your `init` closure currently must match the error
    /// type returned by your [`State`] methods. This limitation may be lifted
    /// in the future.
    ///
    /// # Errors
    ///
    /// If the [`State`] returns an error from [`update`](State::update), [`draw`](State::draw)
    /// or [`event`](State::event), the game will stop running and this method will
    /// return the error.
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
    ///     // Because GameState::new takes `&mut Context` and returns a `State` implementation
    ///     // wrapped in a `Result`, you can use it without a closure wrapper:
    ///     ContextBuilder::new("Hello, world!", 1280, 720)
    ///         .build()?
    ///         .run(GameState::new)
    /// }
    /// ```
    ///
    pub fn run<S, F, E>(&mut self, init: F) -> result::Result<(), E>
    where
        S: State<E>,
        F: FnOnce(&mut Context) -> result::Result<S, E>,
        E: From<TetraError>,
    {
        let state = &mut init(self)?;

        time::reset(self);

        self.running = true;
        self.window.set_visible(true);

        let mut output = Ok(());

        if let Err(e) = self.game_loop(state) {
            output = Err(e);
        }

        self.running = false;
        self.window.set_visible(false);

        output
    }

    pub(crate) fn game_loop<S, E>(&mut self, state: &mut S) -> result::Result<(), E>
    where
        S: State<E>,
        E: From<TetraError>,
    {
        let mut last_time = Instant::now();

        while self.running {
            let curr_time = Instant::now();
            let diff_time = curr_time - last_time;
            last_time = curr_time;

            // Since we fill the buffer when we create the context, we can cycle it
            // here and it shouldn't reallocate.
            self.time.fps_tracker.pop_front();
            self.time.fps_tracker.push_back(diff_time.as_secs_f64());

            platform::handle_events(self, state)?;

            match self.time.tick_rate {
                Some(tick_rate) => {
                    self.time.delta_time = tick_rate;
                    self.time.accumulator = (self.time.accumulator + diff_time).min(tick_rate * 8);

                    while self.time.accumulator >= tick_rate {
                        state.update(self)?;
                        input::clear(self);

                        self.time.accumulator -= tick_rate;
                    }

                    self.time.delta_time = diff_time;
                }

                None => {
                    self.time.delta_time = diff_time;

                    state.update(self)?;
                    input::clear(self);
                }
            }

            state.draw(self)?;

            #[cfg(not(feature = "disable_auto_redraw"))]
            graphics::present(self);

            // This provides a sensible FPS limit when running without vsync, and
            // avoids CPU usage skyrocketing on some systems.
            thread::sleep(Duration::from_millis(1));
        }

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
    pub(crate) multisampling: u8,
    pub(crate) stencil_buffer: bool,
    pub(crate) high_dpi: bool,
    pub(crate) screen_saver_enabled: bool,
    pub(crate) key_repeat: bool,
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

    /// Sets the number of samples that should be used for multisample anti-aliasing.
    ///
    /// The number of samples that can be used varies between graphics cards - `2`, `4` and `8` are reasonably
    /// well supported. Setting the number of samples to `0` will disable multisampling.
    ///
    /// Note that this setting only applies to the main backbuffer - multisampled canvases can
    /// be created via [`Canvas::builder`](crate::graphics::Canvas::builder).
    ///
    /// Defaults to `0`.
    pub fn multisampling(&mut self, multisampling: u8) -> &mut ContextBuilder {
        self.multisampling = multisampling;
        self
    }

    /// Sets whether or not the window should have a stencil buffer.
    ///
    /// If this is enabled, you can use the stencil functions in the
    /// [`graphics`](crate::graphics) module when rendering to the main backbuffer.
    ///
    /// Note that this setting only applies to the main backbuffer - to create a canvas with
    /// a stencil buffer, use [`Canvas::builder`](crate::graphics::Canvas::builder).
    ///
    /// Defaults to `false`.
    pub fn stencil_buffer(&mut self, stencil_buffer: bool) -> &mut ContextBuilder {
        self.stencil_buffer = stencil_buffer;
        self
    }

    /// Sets whether or not the window should use a high-DPI backbuffer, on platforms
    /// that support it (e.g. MacOS with a retina display).
    ///
    /// Note that you may also need some platform-specific config to enable high-DPI
    /// rendering:
    ///
    /// * On Windows, set [`dpiAware`](https://docs.microsoft.com/en-gb/windows/win32/sbscs/application-manifests#dpiaware)
    ///   to `true/pm` and [`dpiAwareness`](https://docs.microsoft.com/en-gb/windows/win32/sbscs/application-manifests#dpiawareness)
    ///   to `permonitorv2` in your application manifest. This should enable the best behaviour available, regardless of how
    ///   old the user's version of Windows is.
    ///     * The [`embed-resource`](https://crates.io/crates/embed-resource) crate can be used to automate embedding
    ///       an application manifest.
    ///     * Alternatively, you can use the [`SetProcessDPIAware`](https://docs.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setprocessdpiaware)
    ///       or [`SetProcessDpiAwareness`](https://docs.microsoft.com/en-us/windows/desktop/api/shellscalingapi/nf-shellscalingapi-setprocessdpiawareness)
    ///       Windows API functions to change these settings programatically, but Microsoft recommend not to do this.
    /// * On Mac, set [`NSHighResolutionCapable`](https://developer.apple.com/documentation/bundleresources/information_property_list/nshighresolutioncapable)
    ///   to `true` in your Info.plist. This is the default on Catalina and higher.
    ///
    /// Defaults to `false`.
    pub fn high_dpi(&mut self, high_dpi: bool) -> &mut ContextBuilder {
        self.high_dpi = high_dpi;
        self
    }

    /// Sets whether or not the user's screen saver can be displayed while the game is running.
    ///
    /// Defaults to `false`.
    pub fn screen_saver_enabled(&mut self, screen_saver_enabled: bool) -> &mut ContextBuilder {
        self.screen_saver_enabled = screen_saver_enabled;
        self
    }

    /// Sets whether or not key repeat should be enabled.
    ///
    /// Normally, a [`KeyPressed`](crate::Event::KeyPressed) event will only be fired once, when
    /// the key is initially pressed. Enabling key repeat causes `KeyPressed` events to be fired
    /// continuously while the key is held down.
    ///
    /// Defaults to `false`.
    pub fn key_repeat(&mut self, key_repeat: bool) -> &mut ContextBuilder {
        self.key_repeat = key_repeat;
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
    /// bounds of the window. The `delta` field of [`Event::MouseMoved`](crate::lifecycle::Event::MouseMoved)
    /// can then be used to track the cursor's changes in position. This is useful
    /// when implementing control schemes that require the mouse to be able to
    /// move infinitely in any direction (for example, FPS-style movement).
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
    /// * [`TetraError::PlatformError`] will be returned if the context cannot be initialized.
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
            multisampling: 0,
            stencil_buffer: false,
            high_dpi: false,
            screen_saver_enabled: false,
            key_repeat: false,
            show_mouse: false,
            grab_mouse: false,
            relative_mouse_mode: false,
            quit_on_escape: false,
            debug_info: false,
        }
    }
}
