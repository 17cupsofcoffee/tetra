// This example demonstrates how you might go about managing multiple animations,
// and switching between them based on the player's input. Something along these
// lines may be added to Tetra itself later, but for now it's not too hard to
// roll your own!

use tetra::graphics::animation::Animation;
use tetra::graphics::{self, Color, DrawParams, Drawable, Rectangle, Texture, Vec2};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

#[derive(PartialEq)]
enum PlayerState {
    Idle,
    Running,
}

struct PlayerAnimation {
    state: PlayerState,
    idle: Animation,
    running: Animation,
}

impl PlayerAnimation {
    fn new(ctx: &mut Context) -> tetra::Result<PlayerAnimation> {
        let texture = Texture::new(ctx, "./examples/resources/tiles.png")?;

        Ok(PlayerAnimation {
            state: PlayerState::Idle,
            idle: Animation::new(
                // Remember, textures are cheap to clone, as they just point at GPU data.
                texture.clone(),
                Rectangle::row(0.0, 256.0, 16.0, 16.0).take(8).collect(),
                5,
            ),
            running: Animation::new(
                texture.clone(),
                Rectangle::row(0.0, 272.0, 16.0, 16.0).take(8).collect(),
                5,
            ),
        })
    }

    fn current(&self) -> &Animation {
        match self.state {
            PlayerState::Idle => &self.idle,
            PlayerState::Running => &self.running,
        }
    }

    fn current_mut(&mut self) -> &mut Animation {
        match self.state {
            PlayerState::Idle => &mut self.idle,
            PlayerState::Running => &mut self.running,
        }
    }

    fn tick(&mut self) {
        self.current_mut().tick();
    }

    fn set_state(&mut self, state: PlayerState) {
        if self.state != state {
            self.state = state;
            self.current_mut().restart();
        }
    }
}

impl Drawable for PlayerAnimation {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        self.current().draw(ctx, params)
    }
}

struct GameState {
    animation: PlayerAnimation,
    position: Vec2,
    velocity: Vec2,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            animation: PlayerAnimation::new(ctx)?,
            position: Vec2::new(240.0, 160.0),
            velocity: Vec2::new(0.0, 0.0),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.animation.tick();

        if input::is_key_down(ctx, Key::A) {
            self.velocity.x = (self.velocity.x - 0.5).max(-5.0);
        } else if input::is_key_down(ctx, Key::D) {
            self.velocity.x = (self.velocity.x + 0.5).min(5.0);
        } else {
            self.velocity.x -= self.velocity.x.abs().min(0.5) * self.velocity.x.signum();
        }

        self.position += self.velocity;

        if self.velocity.x.abs() > 0.0 {
            self.animation.set_state(PlayerState::Running);
        } else {
            self.animation.set_state(PlayerState::Idle);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::draw(
            ctx,
            &self.animation,
            DrawParams::new()
                .position(self.position)
                .origin(Vec2::new(8.0, 8.0))
                .scale(if self.velocity.x >= 0.0 {
                    Vec2::new(2.0, 2.0)
                } else {
                    Vec2::new(-2.0, 2.0)
                }),
        );

        Ok(())
    }
}

fn main() {
    ContextBuilder::new("Controlling Animations", 480, 320)
        .maximized(false)
        .resizable(false)
        .quit_on_escape(true)
        .run_with(GameState::new);
}
