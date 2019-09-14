use tetra::graphics::{self, Color, DrawParams, Texture, Vec2};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::draw(
            ctx,
            &self.texture,
            DrawParams::new()
                .position(Vec2::new(32.0, 32.0))
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(2.0, 2.0)),
        );

        Ok(())
    }
}

fn main() {
    ContextBuilder::new("Rendering a Texture", 640, 480)
        .quit_on_escape(true)
        .run_with(GameState::new);
}
