# Changelog

All notable changes to this project will be documented in this file, following the format defined at keepachangelog.com. Where a change was contributed via a third-party pull request, the author will be credited.

From 0.4.0 onwards, all breaking changes will be explicitly labelled, to make it easier to assess the impact of upgrading.

This project adheres to Semantic Versioning.

## [Upcoming]

### Changed

* Updated `hashbrown` to 0.8.
* Updated `glow` to 0.6.

## [0.5.0] - 2020-09-02

### Added

* `SoundInstance::state` and `SoundInstance::set_state` have been added, which allow you to check the current state of playback and modify it respectively. ([@puppetmaster-](https://github.com/puppetmaster-) in [#205](https://github.com/17cupsofcoffee/tetra/pull/205))
    * These methods use a new enum called `SoundState`, which represents the possible states that a `SoundInstance` can be in.
* **Breaking:** The position of the mouse relative to the previous motion event can now be obtained via the `delta` field on `Event::MouseMoved`. ([@GGalizzi](https://github.com/GGalizzi) in [#206](https://github.com/17cupsofcoffee/tetra/pull/206))
    * As existing code may have been exhastively pattern matching on `Event::MouseMoved`'s data, this is technically a breaking change.
* The window can now be set to `relative_mouse_mode`, which allows the mouse to move outside of the bounds of the window while still reporting motion events. ([@GGalizzi](https://github.com/GGalizzi) in [#206](https://github.com/17cupsofcoffee/tetra/pull/206))
* Various feature flags have been added, allowing you to shrink your dependency tree by removing unused functionality.

### Changed

* **Breaking**: ICO, TIFF, PNM, DDS/DXT and TGA texture loading is now off by default.
    * Feature flags can be used to re-enable them.
* **Breaking:** `graphics::set_texture` is now private.
    * There was no meaningful way to use this function without access to other private functions, so it has been hidden to avoid confusion.
* Updated `bytemuck` to 1.4.
* **Breaking:** Updated `vek` to 0.12.
    * As Vek is exposed via Tetra's API in the form of the `tetra::math` module, this is potentially a breaking change.

## [0.4.2] - 2020-08-14

### Added

* A `visible_rect` method has been added to `Camera`, which calculates the area of the screen that is currently visible. ([@VictorKoenders](https://github.com/VictorKoenders) in [#201](https://github.com/17cupsofcoffee/tetra/pull/201))

### Changed

* Various improvements have been made to the documentation.
* `Camera::project` and `Camera::unproject` no longer require `Camera::update` to be called to give correct results.
    * This is for consistency with the new `visible_rect` method.
* Textures now use `CLAMP_TO_EDGE` wrapping, to avoid some sampling issues when drawing at non-integer co-ordinates.
    * In the future, it may be made possible to select other wrapping modes.
* Updated `bytemuck` to 1.3.

### Fixed

* The matrix created by a `Camera` now correctly reflects the viewport width and height before the first `update`.

## [0.4.1] - 2020-08-02

### Added

* `ContextBuilder` can now be serialized and deserialized via Serde, if the `serde_support` feature is enabled. ([@puppetmaster-](https://github.com/puppetmaster-) in [#195](https://github.com/17cupsofcoffee/tetra/pull/195))
    * Note that the available settings could change between releases of Tetra (semver permitting). If you need a config file schema that will be stable in the long term, consider making your own and then mapping it to Tetra's API, rather than relying on `ContextBuilder` to not change. 

### Changed

* The `TetraError` and `Event` enums are now marked as `non_exhaustive`.
    * This is not a breaking change, as exaustive matching was already enforced via a hidden enum variant. This change just makes the code/docs/errors clearer, as well as potentially unlocking some compiler optimizations in the future.
* Updated `glow` to 0.5.

## [0.4.0] - 2020-06-24

### Added

* The mouse can now be grabbed by the window. ([@tatref](https://github.com/tatref) in [#184](https://github.com/17cupsofcoffee/tetra/pull/184))
    * This is exposed via the `grab_mouse` method on `ContextBuilder`, and the `is_mouse_grabbed`/`set_mouse_grabbed` functions in the `window` module.

### Changed

* **Breaking:** The text rendering API has been rewritten from scratch.
    * It now uses `ab_glyph` instead of `rusttype`, which allows us to support OTF fonts, and should be faster in general.
    * This also fixes several long-standing bugs with text rendering ([#125](https://github.com/17cupsofcoffee/tetra/issues/125), [#161](https://github.com/17cupsofcoffee/tetra/issues/161), [#180](https://github.com/17cupsofcoffee/tetra/issues/180)).
    * The new API has been written with the requirements of bitmap fonts in mind, and a loader for these will likely be added in a future version.
    * As this API may expand in the future, it has been moved into the `tetra::graphics::text` submodule to avoid cluttering the main `graphics` module.
* Improved the documentation for various types' performance characteristics.
* **Breaking:** Updated `vek` to 0.11.
    * As Vek is exposed via Tetra's API in the form of the `tetra::math` module, this is potentially a breaking change.
* Updated `hashbrown` to 0.8.

### Removed

* **Breaking:** `Font` no longer implements `Default`, and the Deja Vu Sans Mono font is no longer bundled with Tetra ([#174](https://github.com/17cupsofcoffee/tetra/issues/174)).
    * It was previously a little murky whether or not the default font's license needed to be included even when you're not using it, due to the bytes being included in the binary.

### Fixed

* Fixed an issue where gamepad axis ranges were not being correctly mapped from integers to floats.

## [0.3.6] - 2020-05-15

### Added

* A new suite of functions has been added to the `window` module, allowing you to query info about the monitors that are connected to the current device.

### Changed

* The window is now hidden when the game loop is not running. This avoids issues where the window would be displayed before the game has a chance to fully load assets, or to determine ideal rendering sizes. 

### Fixed

* Fixed an issue where OpenGL objects would not be properly unbound when they were dropped.
* Fixed an issue where the OpenGL buffer attributes were not being set correctly. 

## [0.3.5] - 2020-04-25

### Added

* File drag and drop events can now be detected via `Event::FileDropped`.
* The clipboard can now be manipulated via `input::get_clipboard_text` and `input::set_clipboard_text`.
* `input::get_key_modifier_down` and `input::get_key_modifier_up` have been added, allowing for code handling the control, alt and shift keys to be made more compact.
* An `Animation` can now be set to stop playing after all the frames have been displayed, instead of looping. This can either be controlled by the `set_repeating` method, or you can create a non-looping animation directly by calling `Animation::once`.
* `hex` and `try_hex` constructors have been added to `Color`.

### Changed

* Updated `sdl2` to 0.34.

## [0.3.4] - 2020-04-12

### Added

* `Animation` now exposes methods for getting and setting the current frame index, and the amount of time that the current frame has been displayed
for. This can be useful when implementing more complex animation behaviors. ([@VictorKoenders](https://github.com/VictorKoenders) in [#169](https://github.com/17cupsofcoffee/tetra/pull/169))
* Some utility methods have been added to `Rectangle` for getting the co-ordinates of the sides, center and corners.
* A `content_mut` getter has been added to `Text`, allowing the content to be mutated using the standard `String` API (e.g. `push_str`, `clear`, etc.).

## [0.3.3] - 2020-04-04

### Added

* The mouse wheel state can now be queried. ([@VictorKoenders](https://github.com/VictorKoenders) in [#164](https://github.com/17cupsofcoffee/tetra/pull/164))

### Changed

* The internal representation of `Texture` objects has been changed to improve performance.
* Updated `sdl2` to 0.33.
* Updated `hashbrown` to 0.7.
* Updated `image` to 0.23.
* Updated `rodio` to 0.11.

## [0.3.2] - 2020-01-15

### Added

* `Rectangle::intersects`, `Rectangle::contains` and `Rectangle::contains_point` have been added.

### Changed

* Added a missing function parameter to `window::set_mouse_visible`, so that you can actually set the value.
    * This is technically a breaking change, but given that the functionality is completely broken, this will be included in a patch release rather than 0.4.
* Restructured the platform layer to better facilitate new backends in the future.
* Improved docs for the `math` module to make it clearer why a re-export is used.
* Updated `glow` to 0.4.

## [0.3.1] - 2019-12-15

### Fixed

* Fixed an issue with variable timesteps causing an infinite loop.

## [0.3.0] - 2019-12-14

### Added

* The `State` trait now provides an `event` method for hooking into window/input events. This is useful in scenarios where you want to be notified of events rather than polling (for example, reacting to window size changes).
* A `Context` can now be configured to have a variable update rate, if that suits your game/architecture better. This is exposed via the `time_step` method on `ContextBuilder`.
* Several new functions have been added to the `time` module, to support both variable and fixed timesteps.
* Functions for getting and setting vsync have been added to `window`.
* Details of the active graphics device can now be retrieved by calling `graphics::get_device_info`.
* `Shader::from_vertex_string` and `Shader::from_fragment_string` constructors have been added.
* `Color::RED`, `Color::GREEN` and `Color::BLUE` constants have been added.
* `graphics::get_transform_matrix`, `graphics::set_transform_matrix` and `graphics::reset_transform_matrix` has been added, which allows you to apply a transformation to your rendering.
* The `Camera` struct has been added, which provides a simple way of creating a transform matrix.
* Serde support has been added (via the `serde_support` Cargo feature) for the following types:
    * `graphics::Color`
    * `graphics::Rectangle`
    * `input::Key`
    * `input::MouseButton`
    * `input::GamepadButton`
    * `input::GamepadAxis`
    * `input::GamepadStick`
    * Various `math` types, as defined by the `Vek` crate.

### Changed

* Tetra now targets the latest stable Rust compiler, rather than a fixed minimum version. This will hopefully change once Cargo has better functionality for enforcing minimum supported compiler versions - currently it's impossible to make guarentees, as our dependencies can change their minimum versions at will.
* `State::draw` no longer takes the blend factor as a third parameter - instead, you can call the new `time::get_blend_factor` function.
* `Key` and `MouseButton` are now Tetra-specific types, rather than re-exporting the SDL versions. Note that some names have been changed for consistency, and some variants have been removed to simplify the docs.
* `TetraError::Sdl` and `TetraError::OpenGl` have been merged into `TetraError::PlatformError`, since they both represent the scenario where something's gone seriously wrong with the underlying platform.
* `DEFAULT_VERTEX_SHADER` and `DEFAULT_FRAGMENT_SHADER` are now const instead of static.
* Screen scaling has been extracted from the core of the engine, and is now provided via the `ScreenScaler` struct. This allows it to be more flexibly integrated with the rest of your game's rendering.
* Various functions now return errors instead of panicking.
* `TetraError` has been reorganized, so that the errors returned are more descriptive.
* The `glm` module has been renamed to `math`, and the `nalgebra-glm` dependency has been replaced with `vek`. 
* `Vec2` is now exported from `math`, not `graphics`.
* More types can now be passed into shader uniforms via the `UniformValue` trait.
* The graphics device debugging info is now hidden by default. Set the `debug_info` option on `Game` to `true` to bring this back.
* The functions for setting the fullscreen/cursor visibility state have been changed to take booleans, instead of there being multiple functions.
* The `Shader::vertex` and `Shader::fragment` constructors have been renamed to `Shader::from_vertex_file` and `Shader::from_fragment_file`.
* Animations now use a `Duration` to specify the frame length, and as such, they are no longer coupled to your game's tick rate. Call `advance` from your `draw` method to advance the animation's timer.
* Updated `glow` to 0.3.0-alpha3.
* Updated `hashbrown` to 0.6.
* Updated `image` to 0.22.
* Updated `glyph_brush` to 0.6.
* Updated `rodio` to 0.10.

### Removed

* `time::duration_to_f64` and `time::f64_to_duration` have been removed, as the standard library now provides this functionality (`Duration::from_secs_f64` and `Duration::as_secs_f64` respectively).
* `ContextBuilder::tick_rate` has been removed, as `ContextBuilder::time_step` now fulfils the same purpose.
* Removed deprecated sub-modules from `graphics`.
* Removed deprecated `color::BLACK` and `color::WHITE` constants - use `Color::BLACK` and `Color::WHITE` instead.
* Removed deprecated `from_data` constructors - use `from_file_data` instead.
* Removed deprecated `DrawParams::build_matrix` method.
* Removed re-exports of `Animation` and `NineSlice` from `graphics` - from now on this functionality will be accessible via `graphics::animation` and `graphics::ui` respectively.

## [0.2.20] - 2019-07-13

### Changed

* All of the SDL2 code is now localized to a single `platform` module. This is a first step towards decoupling the engine from any particular windowing library.
* The OpenGL backend is now implemented using [glow](https://github.com/grovesNL/glow).
* The public module structure of `graphics` has been simplified, so that only animation and GUI code is grouped into submodules, not 'primitive' types. The existing paths have been deprecated.
* The `BLACK` and `WHITE` color constants are now associated with the type, not the module. The existing constants have been deprecated.
* `Color::rgb`, `Color::rgba` and `Rectangle::new` are now `const fn`.
* Updated `glyph-brush` to 0.5.3.

### Fixed

* `window::is_mouse_visible` now actually returns a value (whoops).

## [0.2.19] - 2019-06-13

### Added

* Textures and canvases now provide a method for setting the texture filtering mode.

### Changed

* Updated `image` to 0.21.2.
* Updated `hashbrown` to 0.5.0.
* Updated `rodio` to 0.9.0.

## [0.2.18] - 2019-05-18

### Added

* The `tetras` example now has sound effects and music.
* There are now constructors for `Color` that take `u8` values. ([@aaneto](https://github.com/aaneto) in [#124](https://github.com/17cupsofcoffee/tetra/pull/124))

### Changed

* Tetra now requires Rust 1.32 or higher. While I personally consider this to be a breaking change and was going to save it for 0.3, a dependency has forced our hand by increasing *their* minimum Rust version in a patch release, breaking 1.31 support for all versions of Tetra :(
* Updated `nalgebra-glm` to 0.4.0.

## [0.2.17] - 2019-05-05

### Added

* An example of how to interpolate between ticks using the `dt` has been added.
* Basic support for gamepad vibration has been added.
* A showcase page has been added to the documentation.

### Changed

* Updated `gl` to 0.12.0.
* Updated `image` to 0.21.1.
* Updated `hashbrown` to 0.3.0.
* Updated `glyph-brush` to 0.5.0. 

### Fixed

* Fixed issue with the backbuffer not being bound on the first frame.
* Disconnecting a gamepad while a button is down no longer causes a panic.

## [0.2.16] - 2019-04-07

### Changed

* Reverted `nalgebra-glm` to 0.2.0 to avoid increasing the minimum Rust version.

## [0.2.15] - 2019-04-07

### Added

* `Animation`, `Text` and `NineSlice` now expose more getters and setters, allowing more of their state to be accessed and manipulated after creation.

### Changed

* The way that `nalgebra-glm` is re-exported has been changed slightly, to make it so we can provide a bit more documentation. This should not have any impact on usage or the public facing API.
* Updated `sdl2` to 0.32.2.
* Updated `nalgebra-glm` to 0.4.0.
* Updated `hashbrown` to 0.2.0.
* Updated `glyph_brush` to 0.4.1.

### Removed

* The workaround for the issues with `rand_core` has been removed, as the underlying issue has been fixed. You may need to `cargo clean` if this causes issues.

## [0.2.14] - 2019-03-30

### Added

* `graphics::set_letterbox_color` allows you to set the color of the letterbox bars shown in certain scaling modes.
* Basic support for off-screen rendering/'render to texture' has been implemented, in the form of the `Canvas` object.
* An `animation_controller` example has been added, showing how to change animations based on the player's input. ([@mgocobachi](https://github.com/mgocobachi) in [#110](https://github.com/17cupsofcoffee/tetra/pull/110))
* A `from_file_data` constructor has been added to `Font`, for consistency with `Texture` and `Sound`.

### Changed

* Alpha blending should now work in a more predictable way. This may need further tweaks later on.
* The renderer now flips drawing automatically when drawing to a framebuffer, due to how OpenGL stores textures. This is similar to how FNA and Love2D handle the same problem.
* The renderer no longer implicitly re-binds shaders after calling `graphics::present`.

### Deprecated

* `Font::from_data` has been deprecated.

## [0.2.13] - 2019-03-05

### Added

* A `from_rgba` constructor has been added to `Texture`.
* `from_file_data` constructors have been added to `Texture` and `Sound`. These function the same as the `from_data` constructors, but are more clearly named to reflect the fact that they expect *encoded* data, not *raw* data.

### Changed

* The `tetras` example has been updated to demonstrate how you could approach adding multiple screens/states to a game.

### Deprecated

* The `from_data` constructors have been deprecated.

## [0.2.12] - 2019-02-07

### Changed

* The renderer has been optimized a bit more.

### Fixed

* The 'color' DrawParams property is now being sent to the shader properly - the last few versions had a bug where the blue level was being sent in place of the green channel.

## [0.2.11] - 2019-02-06

### Fixed

* The audio engine now handles repeats/restarts of a `SoundInstance` more reliably. This was mainly an issue with OGG and MP3 files.

## [0.2.10] - 2019-02-03

### Added

* `from_data` constructors were added to `Texture`, `Font` and `Sound`, allowing them to be constructed from binary data. This is useful if you want to use `include_bytes` to bundle assets into your executable. Note that an equivalent constructor already existed on `Shader`, which can be used in combination with `include_str`. 

### Changed

* The default shaders have been amended to use GLSL 1.50 instead of GLSL 1.30. This seems to be required to get Tetra working on Mac.

## [0.2.9] - 2019-02-03

### Changed

* Some optimizations have been made to the rendering code, mainly to avoid unnecessary allocations. This nearly doubled the performance of the `bunnymark` example in release mode!

## [0.2.8] - 2019-02-01

### Added

* The `time::get_fps` function was added, which returns the current FPS, averaged out over the last few seconds. ([@selimeren](https://github.com/selimeren) in [#96](https://github.com/17cupsofcoffee/tetra/pull/96))

## [0.2.7] - 2019-01-23

### Changed

* We now use the [`hashbrown`](https://github.com/Amanieu/hashbrown) implementation of `HashMap`/`HashSet` instead of the `fnv` hasher. The hope was that this would give a performance boost, but it didn't seem to have any real observable impact :( That said, several of Tetra's dependencies use `hashbrown`, so in the interests of keeping the dependency tree light, we're switching anyway.

### Fixed

* A race condition between Rodio and SDL has been fixed.
* While testing `hashbrown` integration, it was found that the benchmark numbers in the FAQ were slightly inaccurate - this has now been fixed.

## [0.2.6] - 2019-01-20

### Added

* Audio playback has been added, using [Rodio](https://github.com/tomaka/rodio) as the backend!
* A port of the popular 'BunnyMark' benchmark has been added, which can be helpful for comparing relative performance of different versions/configurations of Tetra.
* The documentation has been updated to detail the `sdl2_bundled` and `sdl2_static_link` features.

### Changed

* The code that handles sprite transformations has been rewritten, and is now around 10 times faster than 0.2.5 in debug mode, and twice as fast as 0.2.5 in release mode.

### Deprecated

* The `build_matrix` method on `DrawParams` was meant to be an internal utility, not a part of the public API. Tetra no longer uses it, so it has been deprecated, and will be removed in 0.3.

## [0.2.5] - 2019-01-06

### Added

* Custom shaders can now be used for rendering!

### Fixed

* The parameters contained within an instance of `DrawParams` are now publicly accessible - without these, it wasn't possible to write a proper custom implementation of `Drawable`.
* Shaders now bind their outputs explicitly - this should help with compatability.

## [0.2.4] - 2019-01-04

### Fixed

* Fixed an issue where the OpenGL context would fail to initialize when using NVidia's proprietary Linux drivers.

## [0.2.3] - 2019-01-03

### Added

* Tetra now has support for gamepads! The API is roughly the same as that of keyboard and mouse, aside from having to specify which gamepad's state you're querying.

### Changed

* Text is now drawn using the same shader as everything else - this likely won't be noticable now, but it will make things a lot easier once custom shaders get added!

### Fixed

* Some subtle issues around font cache invalidation have been fixed - in general we now let `glyph-brush` handle that side of things.
* Texture flipping was broken in 2.0 - this has now been fixed.
* The OpenGL context now explicitly requests a 32 bit color buffer and double buffering.
* Shaders now bind their texture sampler explicitly, which should avoid black screens on some drivers.

## [0.2.2] - 2018-12-24

### Added

* Tetra now has a [website](https://tetra.seventeencups.net), with a tutorial on how to get started using it.
* `run_with` is now less restrictive about what kinds of closure it will accept.

### Changed

* We now always request an OpenGL 3.2 core profile context - this is required for us to support MacOS.

### Removed

* The `TETRA_OPENGL_FORCE_CORE_PROFILE` environment variable has been removed, since we now always force a core profile.

## [0.2.1] - 2018-12-22

### Added

* Shader errors are now properly reported via `TetraError::OpenGl`.

### Changed

* The shader attribute order is now explicitly defined - this fixes an issue with black screens on some drivers.

## [0.2.0] - 2018-12-21

### Added

* `Texture` now has methods to get the width and height.
* The `bundled` and `static-link` features from the `sdl2` crate can now be used through Tetra by enabling the `sdl2_bundled` and `sdl2_static_link` features. ([@VictorKoenders](https://github.com/VictorKoenders) in [#33](https://github.com/17cupsofcoffee/tetra/pull/33))
* New methods have been added to allow iterating over down/pressed/released keys on the keyboard. ([@VictorKoenders](https://github.com/VictorKoenders) in [#35](https://github.com/17cupsofcoffee/tetra/pull/35))
* Text input typed by the user can now be retrieved using the `input::get_text_input` function. ([@VictorKoenders](https://github.com/VictorKoenders) in [#36](https://github.com/17cupsofcoffee/tetra/pull/36))
* `Text` now has a method for efficiently calculating (and caching) the outer bounds of the text. ([@VictorKoenders](https://github.com/VictorKoenders) in [#41](https://github.com/17cupsofcoffee/tetra/pull/41))
* New methods have been added to `Animation`, allowing it to be modified after it is initially created. ([@VictorKoenders](https://github.com/VictorKoenders) in [#48](https://github.com/17cupsofcoffee/tetra/pull/48))
* There are now numerous different `ScreenScaling` types that can be chosen from.
* Extra options have been added to the `ContextBuilder`, allowing you to start the window in various different states (e.g. fullscreen).
* There are now many new methods for manipulating the window/game loop in the `window` module.
* The `update` and `draw` methods on `State` are now both optional.
* The `graphics` module now re-exports `Vec2`.
* In addition to the normal `run` method, there is now also a `run_with` method that uses a closure to construct the `State`. This is handy when method chaining - see the examples for how it can be used.
* Public types now implement `Debug` and `Clone` where appropriate.
* `TetraError` now implements the standard library `Error` trait.


### Changed

* The library has been upgraded to the 2018 edition of Rust.
* `ContextBuilder::new` now takes the title and size as parameters. The old behavior of the function can be replicated by using `ContextBuilder::default` instead.
* `run` is now a method on `Context`, instead of a free function.
* The `update` and `draw` methods on `State` now return `tetra::Result`, allowing errors to be returned (or propagated via the `?` operator). Any errors returned from these methods will stop the game - your main method can then handle the error (e.g. log it out).
* The `scale` option on `ContextBuilder` has been renamed to `window_scale`, to better reflect its behavior.
* `Shader::from_file` is now called `Shader::new`, and `Shader::new` is now called `Shader::from_string`. This is more consistent with the other constructors.
* Tick rates are now specified in ticks per second.
* The `ContextBuilder` no longer consumes itself when called - this is more flexible for e.g. calling methods inside a conditional.
*  `quit` has been moved to the `window` module.
* `set_tick_rate` has been moved to the `time` module.
* The functions for getting the game's internal width/height have been renamed to disambiguate them from the functions for getting the window width/height.
* Matching on `TetraError` will now force you to add a wildcard arm. This will prevent the addition of new error types from being a breaking change.
* `Shader::from_string` now returns `Result`, as proper error handling will be added to to it eventually.

### Fixed

* The model matrix is now calculated once per `Drawable`, instead of once per vertex. This should speed up rendering.
* The top left corner of a `NineSlice` no longer gets distorted if the x and y of the `fill_rect` aren't equal.
* The renderer now automatically flushes instead of panicking if it hits capacity.
* The renderer will now batch up to 2048 sprites, instead of 1024.
* The default shaders have been rewritten in an older/more compatible syntax, in order to fix some issues with black screens on Mesa drivers.
* The `is_mouse_button_pressed` and `is_mouse_button_released` functions now work correctly. 

## [0.1.6] - 2018-12-09

### Added

* The `Font` and `Text` types have been added, allowing you to render out text using a TTF font.
* Inspired by FNA, the `TETRA_OPENGL_FORCE_CORE_PROFILE` environment variable can now be set to force the application to run using the 3.2 core profile. This might end up getting removed in favour of a more robust solution later on, but it's handy for testing (e.g. Renderdoc requires the core profile to be enabled).

### Fixed

* The internal framebuffer is now an RGB texture instead of an RGBA texture - this was causing some strange issues with blending.

## [0.1.5] - 2018-12-08

### Fixed

* The batcher was performing a flush after texture switches occured, not before.

## [0.1.4] - 2018-12-08

### Added

* Graphics can now be rotated using the `rotation` method on `DrawParams`.

### Fixed

* The calculation of how many elements to render when flushing was broken, which could lead to geometry persisting between frames even when the associated graphic was no longer active.

## [0.1.3] - 2018-12-07

### Added

* The `NineSlice` type has been added, allowing you to easily create dialog boxes from small textures.
* The window size can now be set explicitly. This will take precedence over the scale setting.
* `tetra::error::Result` and `tetra::error::TetraError` are now re-exported in the root of the crate. This allows you to write `tetra::Result` in your function signatures, which aligns a bit better with other custom `Result` types like `io::Result`.
* [An example of how to use the `Animation` type has been added](https://github.com/17cupsofcoffee/tetra/blob/main/examples/animation.rs).


## [0.1.2] - 2018-12-03

### Fixed

* Quick fix to the docs for the mouse button methods.

## [0.1.1] - 2018-12-03

### Added

* Functions for checking the state of the mouse buttons have been added.

### Fixed

* Scaling is now applied relative to the origin.
* Mouse positions now take into account letterboxing.
* Various fixes to the documentation and crate metadata.

## [0.1.0] - 2018-12-02

### Added

* Initial release!

[Upcoming]: https://github.com/17cupsofcoffee/tetra/compare/0.5.0..HEAD
[0.5.0]: https://github.com/17cupsofcoffee/tetra/compare/0.4.2..0.5.0
[0.4.2]: https://github.com/17cupsofcoffee/tetra/compare/0.4.1..0.4.2
[0.4.1]: https://github.com/17cupsofcoffee/tetra/compare/0.4.0..0.4.1
[0.4.0]: https://github.com/17cupsofcoffee/tetra/compare/0.3.6..0.4.0
[0.3.6]: https://github.com/17cupsofcoffee/tetra/compare/0.3.5..0.3.6
[0.3.5]: https://github.com/17cupsofcoffee/tetra/compare/0.3.4..0.3.5
[0.3.4]: https://github.com/17cupsofcoffee/tetra/compare/0.3.3..0.3.4
[0.3.3]: https://github.com/17cupsofcoffee/tetra/compare/0.3.2..0.3.3
[0.3.2]: https://github.com/17cupsofcoffee/tetra/compare/0.3.1..0.3.2
[0.3.1]: https://github.com/17cupsofcoffee/tetra/compare/0.3.0..0.3.1
[0.3.0]: https://github.com/17cupsofcoffee/tetra/compare/0.2.20..0.3.0
[0.2.20]: https://github.com/17cupsofcoffee/tetra/compare/0.2.19..0.2.20
[0.2.19]: https://github.com/17cupsofcoffee/tetra/compare/0.2.18..0.2.19
[0.2.18]: https://github.com/17cupsofcoffee/tetra/compare/0.2.17..0.2.18
[0.2.17]: https://github.com/17cupsofcoffee/tetra/compare/0.2.16..0.2.17
[0.2.16]: https://github.com/17cupsofcoffee/tetra/compare/0.2.15..0.2.16
[0.2.15]: https://github.com/17cupsofcoffee/tetra/compare/0.2.14..0.2.15
[0.2.14]: https://github.com/17cupsofcoffee/tetra/compare/0.2.13..0.2.14
[0.2.13]: https://github.com/17cupsofcoffee/tetra/compare/0.2.12..0.2.13
[0.2.12]: https://github.com/17cupsofcoffee/tetra/compare/0.2.11..0.2.12
[0.2.11]: https://github.com/17cupsofcoffee/tetra/compare/0.2.10..0.2.11
[0.2.10]: https://github.com/17cupsofcoffee/tetra/compare/0.2.9..0.2.10
[0.2.9]: https://github.com/17cupsofcoffee/tetra/compare/0.2.8..0.2.9
[0.2.8]: https://github.com/17cupsofcoffee/tetra/compare/0.2.7..0.2.8
[0.2.7]: https://github.com/17cupsofcoffee/tetra/compare/0.2.6..0.2.7
[0.2.6]: https://github.com/17cupsofcoffee/tetra/compare/0.2.5..0.2.6
[0.2.5]: https://github.com/17cupsofcoffee/tetra/compare/0.2.4..0.2.5
[0.2.4]: https://github.com/17cupsofcoffee/tetra/compare/0.2.3..0.2.4
[0.2.3]: https://github.com/17cupsofcoffee/tetra/compare/0.2.3..0.2.3
[0.2.2]: https://github.com/17cupsofcoffee/tetra/compare/0.2.3..0.2.2
[0.2.1]: https://github.com/17cupsofcoffee/tetra/compare/0.2.3..0.2.1
[0.2.0]: https://github.com/17cupsofcoffee/tetra/compare/0.1.6..0.2.0
[0.1.6]: https://github.com/17cupsofcoffee/tetra/compare/0.1.5..0.1.6
[0.1.5]: https://github.com/17cupsofcoffee/tetra/compare/0.1.4..0.1.5
[0.1.4]: https://github.com/17cupsofcoffee/tetra/compare/0.1.3..0.1.4
[0.1.3]: https://github.com/17cupsofcoffee/tetra/compare/0.1.2..0.1.3
[0.1.2]: https://github.com/17cupsofcoffee/tetra/compare/0.1.1..0.1.2
[0.1.1]: https://github.com/17cupsofcoffee/tetra/compare/0.1.0..0.1.1
[0.1.0]: https://github.com/17cupsofcoffee/tetra/compare/680304..0.1.0
