# FAQ

## General

### Will Tetra be written in pure Rust eventually?

Probably not - SDL2 is a stable and well-tested foundation for building games, and it runs on basically every platform under the sun, so I'm hesitant to replace it. That's likely to remain the only non-Rust dependency, however.

If you're looking for a similar engine that _is_ pure Rust, [GGEZ](https://github.com/ggez/ggez) and [Macroquad](https://github.com/not-fl3/macroquad) are good options.

### Do I have to install SDL manually?

It's possible to have your project automatically compile SDL2 from source as part of the build process. To do so, specify your dependency on Tetra like this:

```toml
[dependencies.tetra]
version = "0.7"
features = ["sdl2_bundled"]
```

This is more convienent, but does however require you to have various build tools installed on your machine (e.g. a C compiler, CMake, etc). In particular, this can be a pain on Windows - hence why it's not the default!

### Can I static link SDL?

If you want to avoid your users having to install SDL2 themselves (or you having to distribute it as a DLL), you can specify for it to be statically linked:

```toml
[dependencies.tetra]
version = "0.7"
features = ["sdl2_static_link"]
```

This comes with some trade-offs, however - make sure you read [this document](https://hg.libsdl.org/SDL/file/default/docs/README-dynapi.md) in the SDL2 repository so that you understand what you're doing!

## Graphics

### Why am I getting a black screen?

Tetra currently targets OpenGL 3.2, so if your hardware does not support this, you might have trouble running games written with the framework. You can check your version of OpenGL by enabling Tetra's debug output - add `.debug_info(true)` to your `ContextBuilder`, run your game, and then look at the console.

If your OpenGL version is 3.2 or higher and you're still getting a black screen, that may indicate a bug - I currently only have access to a Windows machine with a reasonably modern graphics card, so it's not outside the realms of possibility that something that works for me might be broken for others! Please submit an issue, and I'll try to fix it and release a patch version.

### Does Tetra support drawing custom meshes?

Yes - the `Mesh` API has been available since 0.5.4.

### Does Tetra support drawing primitive shapes?

Yes - since 0.5.8, `Mesh` has constructors for basic shapes and there is a `GeometryBuilder` type that can be used to build more complex shapes without manipulating raw vertex data. For more complex/custom use cases, the third party [`lyon`](https://github.com/nical/lyon) crate can be used to generate vertex data.

Note that primitive shape drawing currently isn't batched, so drawing too many generated shapes at once may be slow.

If that's all too complicated and you just want to draw simple rectangles, you could also [create a solid colored `Texture`](https://docs.rs/tetra/0.7.0/tetra/graphics/struct.Texture.html#method.from_data) and then draw that. If you create a 1x1 solid white texture, you can use the `scale` and `color` `DrawParams` to draw multiple rectangles of varying sizes/colors/transparencies in a single draw call.

### Does Tetra support high-DPI rendering?

Tetra added basic support for high-DPI rendering in 0.5.4, which can be enabled via [`ContextBuilder::high_dpi`](https://docs.rs/tetra/0.7/tetra/struct.ContextBuilder.html#method.high_dpi). You may also need some platform-specific configuration/packaging - see the docs for `ContextBuilder::high_dpi` for more info.

## Performance

### Why is my game running slow?

Cargo builds projects in debug mode by default. This can lead to your game running slowly, as the compiler does not fully optimize your code.

Optimizations can be enabled by passing `--release` when building/running your project, but this increases build times quite significantly and removes debug info from the binary, meaning you cannot easily debug or profile your code.

To work around this, add one of the following snippets to your `Cargo.toml`:

```toml
# To enable optimizations in debug mode for Tetra only (requires Rust 1.41):
[profile.dev.package.tetra]
opt-level = 3

# To enable optimizations in debug mode for all dependencies (requires Rust 1.41):
[profile.dev.package."*"]
opt-level = 3

# To enable optimizations in debug mode for the entire project (works with all Rust versions):
[profile.dev]
opt-level = 3
```

Choosing one of the first two options is preferred, as they will not slow down rebuilds of your game's code.

You should also make sure to build with `--release` when distributing your game, so that the final binary is as fast as possible.

#### Benchmarks

The impact of compiler optimizations can be observed by running the `bunnymark` example both with and without the `--release` flag. This example adds 100 new sprites to the screen every time the user clicks, until rendering conistently drops below 60fps.

These were the results when I ran it against Tetra 0.2.9 on my local machine:

| Configuration | Bunnies Rendered |
| ------------- | ---------------- |
| Debug         | 3200             |
| Release       | 230000           |

For reference, my system specs are:

- CPU: AMD Ryzen 5 1600 3.2GHz
- GPU: NVidia GeForce GTX 1050 Ti
- RAM: 8GB DDR4

## Miscellaneous

### Why is it called Tetra?

I'm terrible at naming projects, and [this](https://www.youtube.com/watch?v=g3xg28yaZ5E) happened to be playing when I was typing `cargo new`. I wish there was a better origin story than that :D
