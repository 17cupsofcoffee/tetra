# Tetra

[![Build Status](https://travis-ci.org/17cupsofcoffee/tetra.svg?branch=master)](https://travis-ci.org/17cupsofcoffee/tetra)
[![Crates.io](https://img.shields.io/crates/v/tetra.svg)](https://crates.io/crates/tetra)
[![Minimum Rust Version](https://img.shields.io/badge/minimum%20rust%20version-1.32-orange.svg)](https://www.rust-lang.org/tools/install)
[![Documentation](https://docs.rs/tetra/badge.svg)](https://docs.rs/tetra)
[![License](https://img.shields.io/crates/l/tetra.svg)](LICENSE)
[![Gitter chat](https://badges.gitter.im/tetra-rs/community.png)](https://gitter.im/tetra-rs/community)

Tetra is a simple 2D game framework written in Rust. It uses SDL2 for event handling and OpenGL 3.2+ for rendering.

**Note that Tetra is still extremely early in development!** It may/will have bugs and missing features. That said, you're welcome to give it a go and let me know what you think :)

* [Website/Tutorial](https://tetra.seventeencups.net) 
* [API Docs](https://docs.rs/tetra)
* [FAQ](https://tetra.seventeencups.net/FAQ)

## Features

* XNA/MonoGame-inspired API
* Efficient 2D rendering, with draw call batching by default
* Simple input handling
* Animations/spritesheets
* TTF font rendering
* Multiple screen scaling algorithms, including pixel-perfect variants (for those chunky retro pixels)
* Deterministic game loop, à la [Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)

## Installation

To add Tetra to your project, add the following line to your `Cargo.toml` file:

```
tetra = "0.2"
```

Tetra currently requires Rust 1.32 or higher.

You will also need to install the SDL2 native libraries - full details are provided in the [documentation](https://tetra.seventeencups.net/tutorial/installation.html). 

## Examples

To get a simple window displayed on screen, the following code can be used:

```rust
use tetra::graphics::{self, Color};
use tetra::{Context, ContextBuilder, State};

struct GameState;

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        // Cornflour blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Hello, world!", 1280, 720)
        .build()?
        .run(&mut GameState)
}
```

You can see this example in action by running `cargo run --example hello_world`.

The full list of examples is available [here](https://github.com/17cupsofcoffee/tetra/tree/master/examples).

## Support/Feedback

As mentioned above, Tetra is fairly early in development, so there's likely to be bugs/flaky docs/general weirdness. Please feel free to open an issue/PR if you find something, or leave a question in the [Gitter chat](https://gitter.im/tetra-rs/community).

You can also contact me via [Twitter](https://twitter.com/17cupsofcoffee), or find me lurking in the #gamedev channel on the [Rust Community Discord](https://bit.ly/rust-community).
