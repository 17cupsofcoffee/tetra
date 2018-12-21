# FAQ

## General

### Will Tetra be written in pure Rust eventually?

Probably not - SDL2 is a stable and well-tested foundation for building games, and it runs on basically every platform under the sun, so I'm hesitant to replace it. That's likely to remain the only non-Rust dependency, however.

If you're looking for a similar engine that *is* working towards being pure Rust, [GGEZ](https://github.com/ggez/ggez) and [Quicksilver](https://github.com/ryanisaacg/quicksilver) are excellent options.

### Why is it called Tetra?

I'm terrible at naming projects, and [this](https://www.youtube.com/watch?v=g3xg28yaZ5E) happened to be playing when I was typing `cargo new`. I wish there was a better origin story than that :D

## Compatibility

### Why am I getting a black screen?

First, check the debug output on the console to see which OpenGL version your drivers are using - Tetra primarily aims to support 3.2 and above, and explicitly will **not** work with anything lower than 3.0.

If your OpenGL version is higher than 3.0 and you're still getting a black screen, that may indicate a bug - I currently only have access to a Windows machine with a reasonably modern graphics card, so it's not outside the realms of possibility that something that works for me might be broken for others! Please submit an issue, and I'll try to fix it and release a patch version.

## Performance

### Why is my game running slow?

Rust's performance isn't great in debug mode by default, so if you haven't tweaked your optimization settings, Tetra can get CPU-bound quite quickly. [Other Rust game engines run into this issue too](https://github.com/ggez/ggez/blob/master/docs/FAQ.md#imagesound-loading-and-font-rendering-is-slow).

If your framerate is starting to drop, either run the game in release mode (`cargo run --release`) or set the debug `opt-level` to something higher than zero in your `Cargo.toml`:

```toml
[profile.dev]
opt-level = 1
```
