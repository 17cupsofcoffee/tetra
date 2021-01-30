//! By default, Tetra runs updates at a fixed tick rate, decoupled from the speed at which
//! the screen is rendering. This means your game's physics will behave consistently regardless
//! of how powerful the player's device is, which is nice! However, it does mean that if the
//! update and render rates don't line up nicely, your game might look a bit choppy.
//!
//! There are two ways of handling this:
//!
//! * Interpolation
//!   * Store both the previous state and the current state, and render somewhere in between the two
//!   * Pros: Accurate, no guesswork
//!   * Cons: Complex, introduces a tiny bit of input lag as you're updating one frame ahead of rendering
//!
//! * Extrapolation
//!   * Store only the current state, and guess what the next state will look like
//!   * Pros: Simple to implement, doesn't cause lag
//!   * Cons: Looks weird when the extrapolated state doesn't end up matching reality
//!
//! The example below shows how to implement very naive forms of these techniques, with the
//! tick rate set extremely low for demonstration purposes.
//!
//! For more information, see these articles:
//!
//! * https://gafferongames.com/post/fix_your_timestep/
//! * http://gameprogrammingpatterns.com/game-loop.html
//!
//! If you don't care about any of this, and just want to use an old-school 'lockstep' gameloop, set the
//! `timestep` of your `Context` to `Timestep::Variable`. You will then need to use `time::get_delta_time`
//! in your update and render methods.

use tetra::graphics::{self, Color, Texture};
use tetra::math::Vec2;
use tetra::time::{self, Timestep};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    velocity: Vec2<f32>,

    position_none: Vec2<f32>,
    position_ex: Vec2<f32>,
    position_in_prev: Vec2<f32>,
    position_in_curr: Vec2<f32>,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
            velocity: Vec2::new(16.0, 0.0),

            position_none: Vec2::new(16.0, 16.0),
            position_ex: Vec2::new(16.0, 32.0),
            position_in_prev: Vec2::new(16.0, 48.0),
            position_in_curr: Vec2::new(16.0, 48.0),
        })
    }
}

impl State for GameState {
    fn update(&mut self, _: &mut Context) -> tetra::Result {
        // Without special handling, or with extrapolation, we can just
        // update normally.
        self.position_none += self.velocity;
        self.position_ex += self.velocity;

        // For interpolation, we have to store the previous state as well.
        // We're effectively running the simulation 1 tick ahead of the
        // renderer.
        self.position_in_prev = self.position_in_curr;
        self.position_in_curr += self.velocity;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        // `blend_factor` is a number between 0.0 and 1.0 which represents
        // how far between ticks we currently are. For example, 0.0 would
        // mean the update just ran, 0.99 would mean another update is
        // about to run.
        let blend_factor = time::get_blend_factor(ctx);

        // No special handling - looks choppy!
        self.texture.draw(ctx, self.position_none);

        // With extrapolation - just guess where the position should be
        // based on the object's velocity.
        self.texture
            .draw(ctx, self.position_ex + (self.velocity * blend_factor));

        // With interpolation - we draw at a fixed point between the
        // two stored states.
        self.texture.draw(
            ctx,
            Vec2::lerp(self.position_in_prev, self.position_in_curr, blend_factor),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Interpolation", 640, 480)
        .timestep(Timestep::Fixed(5.0))
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
