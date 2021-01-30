// using sprites by 0x72: https://0x72.itch.io/16x16-industrial-tileset

use std::time::Duration;

use tetra::graphics::animation::Animation;
use tetra::graphics::{self, Color, DrawParams, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    animation: Animation,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            animation: Animation::new(
                Texture::new(ctx, "./examples/resources/tiles.png")?,
                Rectangle::row(0.0, 272.0, 16.0, 16.0).take(8).collect(),
                Duration::from_secs_f64(0.1),
            ),
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        self.animation.advance(ctx);

        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        self.animation.draw(
            ctx,
            DrawParams::new()
                .position(Vec2::new(240.0, 160.0))
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(2.0, 2.0)),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Displaying an Animation", 480, 320)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
