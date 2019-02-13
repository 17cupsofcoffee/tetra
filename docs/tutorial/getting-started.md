# Getting Started

Once you have [installed the native dependencies and set up your project](./installation.md), you're ready to start writing a game!

## Creating Some State

The first step is to create a struct to hold your game's state. To begin with, let's create some text, and store a position where we want to render it:

```rust ,noplaypen
use tetra::graphics::{Text, Font, Vec2};

struct GameState {
    text: Text,
    position: Vec2,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            text: Text::new("Hello, world!", Font::default(), 16.0),
            position: Vec2::new(0.0, 0.0),
        }
    }
}
```

## Adding Some Logic

Now that we have some data, we need a way to manipulate it. In Tetra, you do this by implementing the `State` trait.

`State` has two methods - `update`, which is where you write your game logic, and `draw`, which is where you draw things to the screen. By default, the former is called 60 times a second, and the latter is called in sync with your monitor's refresh rate.

Let's write some code that draws our text moving across the screen:

```rust ,noplaypen
use tetra::graphics::{self, Color};
use tetra::{State, Context};

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.position.x += 1.0;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, self.position);

        Ok(())
    }
}
```

You might have a few questions after reading that code:

* What's that `Context` object that we're passing around? Where does it come from?
* Why do we return `Ok(())` from the methods?

To answer these, we'll need to write our program's `main` function, and actually start our game!

## Building a Context

`Context` is the object that represents all the global state in the framework (the window settings, the rendering engine, etc.). Most functions provided by Tetra will require you to pass the current context, so that they can read from/write to it. As your game grows, you'll probably write your own functions that pass around `Context`, too.

Let's build a new context with a window size of 1280 by 720, and run an instance of our `GameState` struct on it:

```rust ,noplaypen
use tetra::ContextBuilder;

fn main() -> tetra::Result {
    ContextBuilder::new("My First Tetra Game", 1280, 720)
        .build()?
        .run(&mut GameState::new())
}
```

Note that both our `main` function and the `run` method return `tetra::Result`, just like our `update` and `draw` did. If we'd returned an error from `update` or `draw` instead of `Ok(())`, the game would stop running, and `run` would return that error to be handled or logged out. In our case, we just pass it on as `main`'s return value too - Rust will automatically print errors in this case.

If you run `cargo run` from the command line, you should now see your text scrolling across the screen!

## Next Steps

In [the next chapter](./loading-a-texture.md), we'll try loading a texture to display on the screen.

Here's the full example from this chapter:

```rust ,noplaypen
use tetra::graphics::{self, Color, Text, Font, Vec2};
use tetra::{State, Context, ContextBuilder};

struct GameState {
    text: Text,
    position: Vec2,
}

impl GameState {
    fn new() -> GameState {
        GameState {
            text: Text::new("Hello, world!", Font::default(), 16.0),
            position: Vec2::new(0.0, 0.0),
        }
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.position.x += 1.0;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, self.position);

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("My First Tetra Game", 1280, 720)
        .build()?
        .run(&mut GameState::new())
}
```
