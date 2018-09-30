# Tetra

Tetra is a simple 2D game framework written in Rust. It uses SDL 2 for event handling and OpenGL 3.2+ for rendering.

This isn't really intended for general use (yet, at least) - it's just my attempt at pulling out the boilerplate code from a couple of other projects I've worked on, so that I can get up and running faster in the future. As such, I make no guarantees about the API or documentation. Here be dragons, etc. That said, hopefully it'll be helpful reference if you're planning on writing something similar yourself!

If you're looking for a more stable framework to use, [GGEZ](https://github.com/ggez/ggez/), [Amethyst](https://www.amethyst.rs) or [Quicksilver](https://github.com/ryanisaacg/quicksilver/) are probably your best bets right now.

## Design Goals

* Much like GGEZ takes inspiration from Love2D's API, I'm aiming to land somewhere similar to the XNA/MonoGame API.
* I'm *not* aiming to use pure Rust for the backend at the moment, as a lot of the libraries are still under heavy development. I'll let the other frameworks blaze that trail for now :)
