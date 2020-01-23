# Adding the Paddles

In the previous chapter, we created a window and gave it a background color. Next, let's draw some paddles and make them move!

First up, let's update our imports with the new types/modules that we'll be using:

```rust ,noplaypen
use tetra::graphics::{self, Color, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};
```

## Loading a Texture

For our game, we'll be using [some public domain sprites by Kenney](https://www.kenney.nl/assets/puzzle-pack).

Create a folder called `resources` in your project directory, and save this image as `player1.png` inside it:

<div style="text-align: center">
    <img src="./player1.png" alt="Player 1 sprite">
</div>

> The naming of this folder isn't something that's enforced by Tetra - structure your projects however you'd like!

To add this image to our game, we can use our first new type of the chapter: [`Texture`](https://docs.rs/tetra/0.3/tetra/graphics/struct.Texture.html). This represents a piece of image data that has been loaded into graphics memory.

Since we want our texture to stay loaded until the game closes, let's add it as a field in our `GameState` struct:

```rust ,noplaypen
struct GameState {
    paddle_texture: Texture,
}
```

We can then use [`Texture::new`](https://docs.rs/tetra/0.3/tetra/graphics/struct.Texture.html#method.new) to load the sprite and populate that field:

```rust ,noplaypen
fn main() -> tetra::Result {
    ContextBuilder::new("Pong", 640, 480)
        .quit_on_escape(true)
        .build()?
        .run(|ctx| {
            let paddle_texture = Texture::new(ctx, "./resources/player1.png")?;
            Ok(GameState { paddle_texture })
        })
}
```

> A `Texture` is effectively just an ID number under the hood. This means that they are very lightweight and cheap to clone - don't tie yourself in knots trying to pass references to them around your application!

Try running the game now - if all is well, it should start up just like it did last chapter. If you get an error message, check that you've entered the image's path correctly!

## Cleaning Up

We've got our texture loaded in, but our `main` function is starting to look a little cluttered. Before we move on, let's clean things up a little by introducing a proper constructor function for our game state:

```rust ,noplaypen
impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let paddle_texture = Texture::new(ctx, "./resources/player1.png")?;
        Ok(GameState { paddle_texture })
    }
}
```

Because the function's signature matches what's expected by `run`, we can now get rid of the closure and pass the function in directly:

```rust ,noplaypen
fn main() -> tetra::Result {
    ContextBuilder::new("Pong", 640, 480)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
```

Much better! This is the conventional style for a Tetra `main` function, and is what you'll see in most of the examples.

While we're here, let's pull our window width and height out into constants, so that we'll be able to use them in our game logic:

```rust ,noplaypen
const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;

fn main() -> tetra::Result {
    ContextBuilder::new("Pong", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
```

> The `i32` casts look a bit silly, but for most of the places we'll be using the constants, it'll be easier to have them as floating point numbers.

With that bit of housekeeping out of the way, let's finally draw something!

## Drawing to the Screen

To draw our texture, we'll need to make use of another function from the [`graphics`](https://docs.rs/tetra/0.3/tetra/graphics/index.html) module - [`graphics::draw`](https://docs.rs/tetra/0.3/tetra/graphics/fn.draw.html]):

```rust ,noplaypen
// Inside `impl State for GameState`:

fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
    graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

    graphics::draw(ctx, &self.paddle_texture, Vec2::new(16.0, 16.0));

    Ok(())
}
```

This will draw the texture to the screen at position `16.0, 16.0`.

> If you look at the docs for [`graphics::draw`](https://docs.rs/tetra/0.3/tetra/graphics/fn.draw.html), you'll notice that the type of the third parameter is actually `Into<DrawParams>`, not `Vec2`.
> 
> When you pass in a `Vec2`, it is automatically converted into a [`DrawParams`](https://docs.rs/tetra/0.3/tetra/graphics/struct.DrawParams.html) struct with the `position` parameter set. If you want to change other parameters, such as the rotation, color or scale, you can construct your own `DrawParams` instead, using `DrawParams::new`.

## Reacting to Input

A static Pong paddle is no fun, though - let's make it so the player can control it with the <kbd>W</kbd> and <kbd>S</kbd> keys.

In order to do this, we'll first need to store the paddle's position as a field on the state struct, so that it persists from frame to frame. While we're at it, we'll also offset the Y co-ordinate so that the paddle is vertically centered at startup:

```rust ,noplaypen
struct GameState {
    paddle_texture: Texture,
    paddle_position: Vec2<f32>,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let paddle_texture = Texture::new(ctx, "./resources/player1.png")?;
        let paddle_position =
            Vec2::new(16.0, (WINDOW_HEIGHT - paddle_texture.height() as f32) / 2.0);

        Ok(GameState {
            paddle_texture,
            paddle_position,
        })
    }
}
```

We can then plug this field into our existing rendering code, so that the texture will be drawn at the stored position:

```rust ,noplaypen
// Inside `impl State for GameState`:

fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
    graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

    graphics::draw(ctx, &self.paddle_texture, &self.paddle_position);

    Ok(())
}
```

We'll also need to add another constant for our paddle's movement speed:

```rust ,noplaypen
const PADDLE_SPEED: f32 = 8.0;
```

Now we're ready to write some game logic!

While we *could* do this in our `draw` method, this is a bad idea for several reasons:

* Mixing up our game logic and our rendering logic isn't great seperation of concerns.
* The `draw` method does not get called at a consistent rate - the timing can fluctuate depending on the speed of the system the game is being run on, leading to subtle differences in behaviour. This is fine for drawing, but definitely not for physics!

Instead, it's time for us to add another method to our [`State`](https://docs.rs/tetra/0.3/tetra/trait.State.html) implementation. The [`update`](https://docs.rs/tetra/0.3/tetra/trait.State.html#method.update) method is called 60 times a second, regardless of how fast the game as a whole is running. This means that even if rendering slows to a crawl, you can still be confident that the code in that method is deterministic.

> This 'fixed-rate update, variable-rate rendering' style of game loop is best explained by Glenn Fiedler's classic '[Fix Your Timestep](https://gafferongames.com/post/fix_your_timestep/)' blog post. If you've used the `FixedUpdate` method in Unity, this should feel pretty familiar!
>
> If you want to change the rate at which updates happen, or switch to a more traditional 'lockstep' game loop, you can do this via the [`timestep` parameter on `ContextBuilder`](https://docs.rs/tetra/0.3/tetra/struct.ContextBuilder.html#method.timestep).

Inside the `update` method, we can use the functions exposed by the [`input`](https://docs.rs/tetra/0.3/tetra/input/index.html) module in order to check the state of the keyboard:

```rust ,noplaypen
// Inside `impl State for GameState`:

fn update(&mut self, ctx: &mut Context) -> tetra::Result {
    if input::is_key_down(ctx, Key::W) {
        self.paddle_position.y -= PADDLE_SPEED;
    }

    if input::is_key_down(ctx, Key::S) {
        self.paddle_position.y += PADDLE_SPEED;
    }

    Ok(())
}
```

Your paddle should now move up when you press <kbd>W</kbd>, and down when you press <kbd>S</kbd>.

## Adding Player Two

At this point, we've seen all of the Tetra functionality required to complete this chapter - all that remains is to add player two's paddle, and wire it up to the <kbd>Up</kbd> and <kbd>Down</kbd> keys.

First, save this image as `player2.png` in your `resources` folder:

<div style="text-align: center">
    <img src="./player2.png" alt="Player 2 sprite">
</div>

We could just duplicate all of the fields in `GameState` to add another object to the screen, but that feels like a bit of a messy solution. Instead, let's create a new struct to hold the common state of a game entity. We'll add some helper methods to this in the next chapter, but for now, it just needs a constructor:

```rust ,noplaypen
struct Entity {
    texture: Texture,
    position: Vec2<f32>,
}

impl Entity {
    fn new(texture: Texture, position: Vec2<f32>) -> Entity {
        Entity { texture, position }
    }
}
```

> It's worth mentioning at this point: this isn't the only way of structuring a game in Rust!
>
> The language lends itself very well to 'data-driven' design patterns, such as [entity component systems](https://en.wikipedia.org/wiki/Entity_component_system), and you'll definitely want to investigate these concepts if you start writing a bigger game. For now though, let's keep things as simple as possible!

Now for the final stretch - let's refactor our existing code to use the new `Entity` struct, and finally add in our second player!

```rust ,noplaypen
struct GameState {
    player1: Entity,
    player2: Entity,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let player1_texture = Texture::new(ctx, "./resources/player1.png")?;
        let player1_position = Vec2::new(
            16.0,
            (WINDOW_HEIGHT - player1_texture.height() as f32) / 2.0,
        );

        let player2_texture = Texture::new(ctx, "./resources/player2.png")?;
        let player2_position = Vec2::new(
            WINDOW_WIDTH - player2_texture.width() as f32 - 16.0,
            (WINDOW_HEIGHT - player2_texture.height() as f32) / 2.0,
        );

        Ok(GameState {
            player1: Entity::new(player1_texture, player1_position),
            player2: Entity::new(player2_texture, player2_position),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_down(ctx, Key::W) {
            self.player1.position.y -= PADDLE_SPEED;
        }

        if input::is_key_down(ctx, Key::S) {
            self.player1.position.y += PADDLE_SPEED;
        }

        if input::is_key_down(ctx, Key::Up) {
            self.player2.position.y -= PADDLE_SPEED;
        }

        if input::is_key_down(ctx, Key::Down) {
            self.player2.position.y += PADDLE_SPEED;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        graphics::draw(ctx, &self.player1.texture, self.player1.position);
        graphics::draw(ctx, &self.player2.texture, self.player2.position);

        Ok(())
    }
}
```

And with that, we're done!

## Next Steps

In this chapter, we learned how to draw textures and read keyboard input, and put that knowledge to good use by creating some Pong paddles. [Next, we'll add the last piece of the puzzle - the ball](./03-adding-a-ball.md).

Here's the code from this chapter in full:

```rust ,noplaypen
use tetra::graphics::{self, Color, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;
const PADDLE_SPEED: f32 = 8.0;

fn main() -> tetra::Result {
    ContextBuilder::new("Pong", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}

struct Entity {
    texture: Texture,
    position: Vec2<f32>,
}

impl Entity {
    fn new(texture: Texture, position: Vec2<f32>) -> Entity {
        Entity { texture, position }
    }
}

struct GameState {
    player1: Entity,
    player2: Entity,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let player1_texture = Texture::new(ctx, "./resources/player1.png")?;
        let player1_position = Vec2::new(
            16.0,
            (WINDOW_HEIGHT - player1_texture.height() as f32) / 2.0,
        );

        let player2_texture = Texture::new(ctx, "./resources/player2.png")?;
        let player2_position = Vec2::new(
            WINDOW_WIDTH - player2_texture.width() as f32 - 16.0,
            (WINDOW_HEIGHT - player2_texture.height() as f32) / 2.0,
        );

        Ok(GameState {
            player1: Entity::new(player1_texture, player1_position),
            player2: Entity::new(player2_texture, player2_position),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_down(ctx, Key::W) {
            self.player1.position.y -= PADDLE_SPEED;
        }

        if input::is_key_down(ctx, Key::S) {
            self.player1.position.y += PADDLE_SPEED;
        }

        if input::is_key_down(ctx, Key::Up) {
            self.player2.position.y -= PADDLE_SPEED;
        }

        if input::is_key_down(ctx, Key::Down) {
            self.player2.position.y += PADDLE_SPEED;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        graphics::draw(ctx, &self.player1.texture, self.player1.position);
        graphics::draw(ctx, &self.player2.texture, self.player2.position);

        Ok(())
    }
}
```