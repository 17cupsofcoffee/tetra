# Changelog

## Unreleased

### New Features

* `Texture` now has methods to get the width and height ([#31](https://github.com/17cupsofcoffee/tetra/issues/31)).
* The `bundled` and `static-link` features from the `sdl2` crate can now be used through Tetra by enabling the `sdl2_bundled` and `sdl2_static_link` features ([#33](https://github.com/17cupsofcoffee/tetra/pull/33), by [@VictorKoenders](https://github.com/VictorKoenders)).
* New methods have been added to allow iterating over down/pressed/released keys on the keyboard ([#35](https://github.com/17cupsofcoffee/tetra/pull/35), by [@VictorKoenders](https://github.com/VictorKoenders)):
    * `input::get_keys_down`
    * `input::get_keys_pressed`
    * `input::get_keys_released`
* Text input typed by the user can now be retrieved using the `input::get_text_input` function ([#36](https://github.com/17cupsofcoffee/tetra/pull/36), by [@VictorKoenders](https://github.com/VictorKoenders)).

### Bug Fixes

* The model matrix is now calculated once per `Drawable`, instead of once per vertex. This should speed up rendering ([#26](https://github.com/17cupsofcoffee/tetra/issues/26)).
* The top left corner of a `NineSlice` no longer gets distorted if the x and y of the `fill_rect` aren't equal.
* The renderer now automatically flushes instead of panicking if it hits capacity ([#30](https://github.com/17cupsofcoffee/tetra/issues/30)).
* The default shaders have been rewritten in an older/more compatible syntax, in order to fix some issues with black screens on Mesa drivers ([#14](https://github.com/17cupsofcoffee/tetra/issues/14)).

## 0.1.6 (December 9, 2018)

### New Features

* The `Font` and `Text` types have been added, allowing you to render out text using a TTF font ([#17](https://github.com/17cupsofcoffee/tetra/issues/17)).
* Inspired by FNA, the `TETRA_OPENGL_FORCE_CORE_PROFILE` environment variable can now be set to force the application to run using the 3.2 core profile. This might end up getting removed in favour of a more robust solution later on, but it's handy for testing (e.g. Renderdoc requires the core profile to be enabled).

### Bug Fixes

* The internal framebuffer is now an RGB texture instead of an RGBA texture - this was causing some strange issues with blending.

## 0.1.5 (December 8, 2018)

### Bug Fixes

* The batcher was performing a flush after texture switches occured, not before.

## 0.1.4 (December 8, 2018)

### New Features

* Graphics can now be rotated using the `rotation` method on `DrawParams` ([#24](https://github.com/17cupsofcoffee/tetra/issues/24)).

### Bug Fixes

* The calculation of how many elements to render when flushing was broken, which could lead to geometry persisting between frames even when the associated graphic was no longer active.

## 0.1.3 (December 7, 2018)

### New Features

* The `NineSlice` type has been added, allowing you to easily create dialog boxes from small textures ([#23](https://github.com/17cupsofcoffee/tetra/issues/23)).
* The window size can now be set explicitly. This will take precedence over the scale setting ([#19](https://github.com/17cupsofcoffee/tetra/issues/19)).
* `tetra::error::Result` and `tetra::error::TetraError` are now re-exported in the root of the crate. This allows you to write `tetra::Result` in your function signatures, which aligns a bit better with other custom `Result` types like `io::Result` ([#18](https://github.com/17cupsofcoffee/tetra/issues/18)).
* [An example of how to use the `Animation` type has been added](https://github.com/17cupsofcoffee/tetra/blob/master/examples/animation.rs)  ([#16](https://github.com/17cupsofcoffee/tetra/issues/16)).


## 0.1.2 (December 3, 2018)

### Bug Fixes

* Quick fix to the docs for the mouse button methods.

## 0.1.1 (December 3, 2018)

### New Features

* Functions for checking the state of the mouse buttons have been added.
    * `input::is_mouse_button_down`
    * `input::is_mouse_button_up`
    * `input::is_mouse_button_pressed`
    * `input::is_mouse_button_released`

### Bug Fixes

* Scaling is now applied relative to the origin ([#12](https://github.com/17cupsofcoffee/tetra/issues/12)).
* Mouse positions now take into account letterboxing ([#13](https://github.com/17cupsofcoffee/tetra/issues/13)).
* Various fixes to the documentation and crate metadata.

## 0.1.0 (December 2, 2018)

Initial release!