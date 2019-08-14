use tetra::graphics::{self, Color, Texture, Vec2};
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
        graphics::draw(ctx, &self.texture, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() {
    ContextBuilder::new("Rendering a Texture", 160, 144)
        .maximized(true)
        .resizable(true)
        .quit_on_escape(true)
        .run_with(GameState::new);
}
