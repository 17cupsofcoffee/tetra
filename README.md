# Tetra

Tetra is a simple 2D game framework written in Rust. It uses SDL 2 for event handling and OpenGL 3.2+ for rendering.

**Note:** This definitely isn't developed enough for general use yet - I'm just putting it on GitHub so that I can get feedback on some of the trickier bits of the codebase! If you're looking to write a game in Rust, [GGEZ](https://github.com/ggez/ggez/), [Amethyst](https://www.amethyst.rs) or [Quicksilver](https://github.com/ryanisaacg/quicksilver/) are probably your best bets right now.

## Design Goals

* Much like GGEZ takes inspiration from Love2D's API, I'm aiming to land somewhere similar to the XNA/MonoGame API.
* I'm *not* aiming to use pure Rust for the backend at the moment, as a lot of the libraries are still under heavy development. I'll let the other frameworks blaze that trail for now :)