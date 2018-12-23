# Getting Started

Once you have [installed SDL and set up your project](./installation.md), you're ready to start writing a game!

## Creating Some State

The first step is to create a struct to hold your game's state. To begin with, let's create some text, and store a position where we want to render it:

```rust
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

```rust
use tetra::graphics;
use tetra::{State, Context};

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.position.x += 1.0;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, self.pos);

        Ok(())
    }
}
```

You might be wondering what the `Context` that we're passing around is for, or where it comes from - let's take a closer look!

## Building a Context

`Context` is the object that represents all the global state in the framework (the window settings, the rendering engine, etc.). Most functions provided by Tetra will require you to pass the current context, so that they can read from/write to it. As your game grows, you'll probably write your own functions that pass around `Context`, too. 

Let's build a new context with a window size of 1280 by 720, and run an instance of our `GameState` struct on it:

```rust
use tetra::ContextBuilder;

fn main() -> tetra::Result {
    ContextBuilder::new("My First Tetra Game", 1280, 720)
        .build()?
        .run(&mut GameState::new())
}
```

If you try `cargo run`, you should see your text scrolling across the screen!

## Next Steps

In [the next chapter](./loading-a-texture.md), we'll try loading a texture to display on the screen. 

Here's the full example from this chapter:

```rust
use tetra::graphics::{self, Text, Font, Vec2};
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