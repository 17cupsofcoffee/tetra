# Building for the Web

> ⚠️ Tetra's support for WebAssembly is still experimental. You may find bugs or things
> that don't quite work the same as on desktop. Please report any issues you find!

Version 0.3 of Tetra comes with support for WebAssembly, which allows you to build your games for the web!

The ecosystem is still fairly new and experimental, so there's a few extra steps you need to take to get
your game ready. In the future, it should all hopefully 'just work'!

## Creating the Entry Point

Unfortunately, `wasm-bindgen` doesn't yet have first-class support for projects with a `main` function.
Thankfully, it's easy to work around this by providing your own entry point.

First, add `wasm-bindgen` to your `Cargo.toml` as a platform-specific dependency:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2.50"
```

Then create an entry point in `main.rs`:

```rust ,noplaypen
// The module is optional, but saves you from having to place cfg attributes
// on each individual item.

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen(start)]
    pub fn wasm_main() {
        super::main();
    }
}
```

## Setting Up Assets

Loading assets is much more complicated on the web than it is on the desktop - they're delivered over
the network, and you can't block the main thread while they load. A proper solution to this will be
added to Tetra in the future, but in the meantime, the easiest option is to bundle your assets into
the executable itself.

To do this, use the raw data constructors, in combination with the standard library's
`include_bytes!` macro:

```rust ,noplaypen
let texture = Texture::from_file_data(ctx, include_bytes!("texture.png"))?;
let sound = Sound::from_file_data(include_bytes!("./sound.wav"));
let font = Font::from_file_data(ctx, include_bytes!("font.ttf"));
let shader = Shader::from_string(ctx, include_bytes!("shader.vert"), include_bytes!("shader.frag"))?;
```

Some assets will need further tweaks to get them to work in the browsers:

### Sounds

The `Sound` API currently doesn't work in the browser. [Please help me fix that!](https://github.com/17cupsofcoffee/tetra/issues/138)

### Shaders

Bear in mind when using custom shaders that WebGL uses GLSL ES, not the standard desktop version of GLSL.
This may require you to modify your shaders to be compatible with both, or to provide different shaders
for both.

## Setting a Panic Hook (optional)

By default, the `wasm32-unknown-unknown` target does not log out the message or the stack trace when the
program panics. Instead, you just get a confusing 'unreachable executed' message. The
`console_error_panic_hook` crate provides a plug-and-play solution to this problem.

To set this up, add the crate as a platform-specific dependency like you did for `wasm-bindgen`,
then call `console_error_panic_hook::set_once()` in your `wasm_main` function.

## Building the Game

Once you've gone through the above steps, you're ready to build for the web!

Run the following commands:

```sh
# Install the wasm-bindgen CLI (first time only)
cargo install wasm-bindgen-cli

# Create an output folder (first time only)
mkdir generated

# Build the initial WASM file
cargo build --target wasm32-unknown-unknown

# Process the built file with wasm-bindgen
wasm-bindgen ./target/wasm32-unknown-unknown/debug/project-name.wasm --out-dir generated --target web
```

Once this is complete, the built assets for your game will be available in `generated` (or whichever `--out-dir` you chose).

## Deploying the Game

To run the game in a browser, you need to create a HTML file that loads the generated JavaScript/WASM:

```html
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width, initial-scale=1.0" />
        <meta http-equiv="X-UA-Compatible" content="ie=edge" />
        <title>Tetra</title>
    </head>
    <body>
        <canvas id="canvas" width="640" height="480"></canvas>

        <!-- Make sure the path to the file is correct! -->
        <script type="module">
            import init from "./generated/project-name.js";
            init();
        </script>
    </body>
</html>
```

Note that WebAssembly cannot be run from a `file://` URL - you will need to spin up a static server
([`miniserve`](https://github.com/svenstaro/miniserve) is nice).

Congratulations, your game is running on the web!

## Next Steps

While this is enough to get you up and running, there's more steps you can take to get your game running faster
and smaller on the web. Make sure you `cargo build` with the `--release` option in production, and take a look at
the Rust WebAssembly working group's guide to [optimizing code size](https://rustwasm.github.io/book/reference/code-size.html).