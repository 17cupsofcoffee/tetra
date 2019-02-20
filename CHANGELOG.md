# Changelog

All notable changes to this project will be documented in this file, following the format defined at keepachangelog.com. Where a change was contributed via a third-party pull request, the author will be credited.

This project adheres to Semantic Versioning.

## Upcoming

### Changed

* The `tetras` example has been updated to demonstrate how you could approach adding multiple screens/states to a game.

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
* [An example of how to use the `Animation` type has been added](https://github.com/17cupsofcoffee/tetra/blob/master/examples/animation.rs).


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
