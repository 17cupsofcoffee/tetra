# Adding a Ball

In the [previous chapter](./02-adding-the-paddles.md), we added paddles to the game - but they've got nothing to hit! Let's finish things off.

As with the other chapters, we'll start by updating our imports:

```rust ,noplaypen
use tetra::graphics::{self, Color, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::window;
use tetra::{Context, ContextBuilder, State};
```

## Creating the Entity

First, you'll need to download the sprite for the ball - as with the paddles, this was [created by Kenney](https://www.kenney.nl/assets/puzzle-pack), and is available in the public domain.

<div style="text-align: center">
    <img src="./ball.png" alt="Ball sprite">
</div>

Next, we'll draw an `Entity` for the ball, positioned in the center of the screen. This is all stuff from the previous chapter - feel free to go back if you need a refresher!

```rust ,noplaypen
// Inside `GameState::new`:

let ball_texture = Texture::new(ctx, "./resources/ball.png")?;
let ball_position = Vec2::new(
    WINDOW_WIDTH / 2.0 - ball_texture.width() as f32 / 2.0,
    WINDOW_HEIGHT / 2.0 - ball_texture.height() as f32 / 2.0,
);

Ok(GameState {
    player1: Entity::new(player1_texture, player1_position),
    player2: Entity::new(player2_texture, player2_position),
    ball: Entity::new(ball_texture, ball_position),
})
```

```rust ,noplaypen
// Inside `impl State for GameState`:

fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
    graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

    graphics::draw(ctx, &self.player1.texture, self.player1.position);
    graphics::draw(ctx, &self.player2.texture, self.player2.position);
    graphics::draw(ctx, &self.ball.texture, self.ball.position);

    Ok(())
}
```

If you run the game now, you should see the ball hanging precariously in mid-air between the two paddles.

## Applying Physics

Unlike our paddles, which are moved directly by the keyboard input, we want our ball to move even when the player's not doing anything. To do this, we'll implement some basic physics.

First, we need to add the relevant info to our `Entity` struct:

```rust ,noplaypen
struct Entity {
    texture: Texture,
    position: Vec2<f32>,
    velocity: Vec2<f32>,
}

impl Entity {
    fn new(texture: Texture, position: Vec2<f32>) -> Entity {
        Entity::with_velocity(texture, position, Vec2::zero())
    }

    fn with_velocity(texture: Texture, position: Vec2<f32>, velocity: Vec2<f32>) -> Entity {
        Entity {
            texture,
            position,
            velocity,
        }
    }
}
```

We'll also need another constant, so that we can tweak the ball's speed later if needed:

```rust ,noplaypen
const BALL_SPEED: f32 = 5.0;
```

We can now set the ball's velocity when the game starts up - we'll make Player One have the first swing:

```rust ,noplaypen
// Inside `GameState::new`:

let ball_velocity = Vec2::new(-BALL_SPEED, 0.0);

Ok(GameState {
    player1: Entity::new(player1_texture, player1_position),
    player2: Entity::new(player2_texture, player2_position),
    ball: Entity::with_velocity(ball_texture, ball_position, ball_velocity),
})
```

Now that our ball knows what its velocity is, we can use that information to move it around. Add the following line to your `update` method, just before the `Ok(())`:

```rust ,noplaypen
self.ball.position += self.ball.velocity;
```

If you run the game now, you should see the ball start moving - and promptly fly through Player One's paddle and off the left hand side of the screen. That seems somewhat unfair on Player One! Time for some basic collision detection.

## Making the Ball Collide

Since all our game objects are vaguely rectangular (even the ball, if you squint hard enough), we can use one of the simplest forms of collision detection: axis-aligned bounding boxes, or AABB for short.

This technique takes a rectangle, and does some extremely simple math to determine if it intersects with another rectangle. It's used so commonly that Tetra has a utility for it out of the box, imaginatively named `Rectangle::intersects`.

Since our collision detection is all `Rectangle` based, let's create some helper methods on `Entity` to give us the entity's bounds in that form:

```rust ,noplaypen
// Inside `impl Entity`:

fn width(&self) -> f32 {
    self.texture.width() as f32
}

fn height(&self) -> f32 {
    self.texture.height() as f32
}

fn bounds(&self) -> Rectangle {
    Rectangle::new(
        self.position.x,
        self.position.y,
        self.width(),
        self.height(),
    )
}
```

Now, at the end of our `update` method, we can check if the ball intersects with either of the paddles, and if so, flip the X component of the velocity:

```rust ,noplaypen
let player1_bounds = self.player1.bounds();
let player2_bounds = self.player2.bounds();
let ball_bounds = self.ball.bounds();

let paddle_hit = if ball_bounds.intersects(&player1_bounds) {
    Some(&self.player1)
} else if ball_bounds.intersects(&player2_bounds) {
    Some(&self.player2)
} else {
    None
};

if paddle_hit.is_some() {
    self.ball.velocity.x = -self.ball.velocity.x;
}
```

> Storing the identity of the paddle that got hit is redundant right now, but we'll use it later!
>
> Also, more experienced gamedevs may notice a potential problem with doing collision detection in this way - if the ball's speed makes it move further than the width of the paddle in one tick, it'll never intersect, making it look like the ball has just phased straight through the paddle!
>
> This phenomenon is known as 'tunnelling', and fixing it is out of scope for this tutorial - feel free to research it yourself, though!

Now our ball bounces between the two paddles - but it never changes height or speed, which makes for a pretty boring game of Pong. Let's add some gameplay!

## Putting Our Own Spin On It

There's a variety of different ways to give the player some control over the ball in a Pong clone. One of the simplest solutions is to vary the angle of the ball's movement based on which part of the paddle was hit - that's what we're going to implement now!

In addition, we want to make sure that the game doesn't last forever - we'll do this by gradually increasing the X velocity of the ball with each bounce.

As before, we'll start by adding a new helper method to `Entity` - this time it'll give us the center point of our object:

```rust ,noplaypen
// Inside `impl Entity`:

fn centre(&self) -> Vec2<f32> {
    Vec2::new(
        self.position.x + (self.width() / 2.0),
        self.position.y + (self.height() / 2.0),
    )
}
```

We'll also go to the top of the file and add some constants:

```rust ,noplaypen
const PADDLE_SPIN: f32 = 4.0;
const BALL_ACC: f32 = 0.05;
```

Now we can replace the `if paddle_hit.is_some()` block with our 'spin' and speedup logic:

```rust ,noplaypen
if let Some(paddle) = paddle_hit {
    // Increase the ball's velocity, then flip it.
    self.ball.velocity.x =
        -(self.ball.velocity.x + (BALL_ACC * self.ball.velocity.x.signum()));

    // Calculate the offset between the paddle and the ball, as a number between
    // -1.0 and 1.0.
    let offset = (paddle.centre().y - self.ball.centre().y) / paddle.height();

    // Apply the spin to the ball.
    self.ball.velocity.y += PADDLE_SPIN * -offset;
}
```

> I'll admit, it's a little bit wasteful to calculate the X center as well, but I'm aiming for code clarity over maximum efficiency. Besides, it's a Pong clone, not Crysis!

Now the player has some agency over where the ball goes - too much agency, as it turns out, as they can just send it flying off the top of the screen! A little bit more code at the end of `update` will fix that:

```rust ,noplaypen
if self.ball.position.y <= 0.0 || self.ball.position.y + self.ball.height() >= WINDOW_HEIGHT
{
    self.ball.velocity.y = -self.ball.velocity.y;
}
```

## Picking a Winner

At this point, we basically have a fully functioning game of Pong! The only thing left to do is declare one player the winner when the other misses a hit.

This part is simple compared to everything else we've done this chapter - just add the following code to the end of your `update` method:

```rust ,noplaypen
if self.ball.position.x < 0.0 {
    window::quit(ctx);
    println!("Player 2 wins!");
}

if self.ball.position.x > WINDOW_WIDTH {
    window::quit(ctx);
    println!("Player 1 wins!");
}
```

And with that, we're finally done! Go find a friend and play some Pong!

## Closing Notes

First of all - if you've been following along, thank you for sticking with this tutorial for the months it's taken me to write it!

While this game is 'complete', there's a lot of ways it could be improved - here's some suggestions for what to try next (ranked from easy to hard):

* Tweak the constants to change how the game feels to play.
* Make the paddles have a velocity, so the player can have more fine-grained control over their movement.
* Add a score counter, and make the field reset after a ball goes offscreen.
* Add some cool effects, or replace the sprites with your own.
* Rewrite the game using an ECS library like [Specs](https://github.com/amethyst/specs), [Legion](https://github.com/TomGillen/legion) or [Hecs](https://github.com/Ralith/hecs).

Finally, here's the full code:

```rust ,noplaypen
use tetra::graphics::{self, Color, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::window;
use tetra::{Context, ContextBuilder, State};

const WINDOW_WIDTH: f32 = 640.0;
const WINDOW_HEIGHT: f32 = 480.0;
const PADDLE_SPEED: f32 = 8.0;
const PADDLE_SPIN: f32 = 4.0;
const BALL_SPEED: f32 = 5.0;
const BALL_ACC: f32 = 0.05;

fn main() -> tetra::Result {
    ContextBuilder::new("Pong", WINDOW_WIDTH as i32, WINDOW_HEIGHT as i32)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}

struct Entity {
    texture: Texture,
    position: Vec2<f32>,
    velocity: Vec2<f32>,
}

impl Entity {
    fn new(texture: Texture, position: Vec2<f32>) -> Entity {
        Entity::with_velocity(texture, position, Vec2::zero())
    }

    fn with_velocity(texture: Texture, position: Vec2<f32>, velocity: Vec2<f32>) -> Entity {
        Entity {
            texture,
            position,
            velocity,
        }
    }

    fn width(&self) -> f32 {
        self.texture.width() as f32
    }

    fn height(&self) -> f32 {
        self.texture.height() as f32
    }

    fn centre(&self) -> Vec2<f32> {
        Vec2::new(
            self.position.x + (self.width() / 2.0),
            self.position.y + (self.height() / 2.0),
        )
    }

    fn bounds(&self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y,
            self.width(),
            self.height(),
        )
    }
}

struct GameState {
    player1: Entity,
    player2: Entity,
    ball: Entity,
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

        let ball_texture = Texture::new(ctx, "./resources/ball.png")?;
        let ball_position = Vec2::new(
            WINDOW_WIDTH / 2.0 - ball_texture.width() as f32 / 2.0,
            WINDOW_HEIGHT / 2.0 - ball_texture.height() as f32 / 2.0,
        );
        let ball_velocity = Vec2::new(-BALL_SPEED, 0.0);

        Ok(GameState {
            player1: Entity::new(player1_texture, player1_position),
            player2: Entity::new(player2_texture, player2_position),
            ball: Entity::with_velocity(ball_texture, ball_position, ball_velocity),
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

        self.ball.position += self.ball.velocity;

        let player1_bounds = self.player1.bounds();
        let player2_bounds = self.player2.bounds();
        let ball_bounds = self.ball.bounds();

        let paddle_hit = if ball_bounds.intersects(&player1_bounds) {
            Some(&self.player1)
        } else if ball_bounds.intersects(&player2_bounds) {
            Some(&self.player2)
        } else {
            None
        };

        if let Some(paddle) = paddle_hit {
            // Increase the ball's velocity, then flip it.
            self.ball.velocity.x =
                -(self.ball.velocity.x + (BALL_ACC * self.ball.velocity.x.signum()));

            // Calculate the offset between the paddle and the ball, as a number between
            // -1.0 and 1.0.
            let offset = (paddle.centre().y - self.ball.centre().y) / paddle.height();

            // Apply the spin to the ball.
            self.ball.velocity.y += PADDLE_SPIN * -offset;
        }

        if self.ball.position.y <= 0.0 || self.ball.position.y + self.ball.height() >= WINDOW_HEIGHT
        {
            self.ball.velocity.y = -self.ball.velocity.y;
        }

        if self.ball.position.x < 0.0 {
            window::quit(ctx);
            println!("Player 2 wins!");
        }

        if self.ball.position.x > WINDOW_WIDTH {
            window::quit(ctx);
            println!("Player 1 wins!");
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        graphics::draw(ctx, &self.player1.texture, self.player1.position);
        graphics::draw(ctx, &self.player2.texture, self.player2.position);
        graphics::draw(ctx, &self.ball.texture, self.ball.position);

        Ok(())
    }
}
```
