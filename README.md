# Tetra

[![Crates.io](https://img.shields.io/crates/v/tetra.svg)](https://crates.io/crates/tetra)
[![Documentation](https://docs.rs/tetra/badge.svg)](https://docs.rs/tetra)
[![License](https://img.shields.io/crates/l/tetra.svg)](LICENSE)

Tetra is a simple 2D game framework written in Rust. It uses SDL2 for event handling and OpenGL 3.2+ for rendering.

**Note that Tetra is still extremely early in development!** It may/will have bugs and missing features (the big ones currently being sound and gamepad support). That said, you're welcome to give it a go and let me know what you think :)

## Features

* XNA/MonoGame-inspired API
* Efficient 2D rendering, with draw call batching by default
* Animations/spritesheets
* Pixel-perfect screen scaling 
* Deterministic game loop, à la [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/).

## Installation

To add Tetra to your project, add the following line to your `Cargo.toml` file:

```
tetra = "0.1"
```

You will also need to install the SDL2 native libraries, as described [here](https://github.com/Rust-SDL2/rust-sdl2#user-content-requirements). The 'bundled' and 'static linking' features described can be activated using the `sdl2_bundled` and `sdl2_static_link` Cargo features in Tetra.

## Examples

To get a simple window displayed on screen, the following code can be used:

```rust
extern crate tetra;

use tetra::graphics::{self, Color};
use tetra::{Context, ContextBuilder, State};

struct GameState;

impl State for GameState {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        // Cornflour blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Hello, world!")
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState;

    tetra::run(ctx, state)
}
```

You can see this example in action by running `cargo run --example hello_world`.

The full list of examples available are:

* [`hello_world`](examples/hello_world.rs) - Opens a window and clears it with a solid color.
* [`texture`](examples/texture.rs) - Loads and displays a texture.
* [`animation`](examples/animation.rs) - Displays an animation, made up of regions from a texture.
* [`text`](examples/text.rs) - Displays text using a TTF font.
* [`nineslice`](examples/nineslice.rs) - Slices a texture into nine segments to display a dialog box.
* [`keyboard`](examples/keyboard.rs) - Moves a texture around based on keyboard input.
* [`mouse`](examples/mouse.rs) - Moves a texture around based on mouse input.
* [`text_input`](examples/text_input.rs) - Displays text as it is typed in by the player.
* [`tetras`](examples/tetras.rs) - A full example game (which is entirely legally distinct from a certain other block-based puzzle game *cough*).

## Support/Feedback

As mentioned above, Tetra is fairly early in development, so there's likely to be bugs/flaky docs/general weirdness. Please feel free to leave an issue/PR if you find something!

You can also contact me via [Twitter](https://twitter.com/17cupsofcoffee), or find me lurking in the #gamedev channel on the [Rust Community Discord](https://bit.ly/rust-community).