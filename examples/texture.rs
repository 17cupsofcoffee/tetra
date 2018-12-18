use tetra::graphics::{self, Color, Texture, Vec2};
use tetra::{self, Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));
        graphics::draw(ctx, &self.texture, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new("Rendering a Texture", 160, 144)
        .maximized(true)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState {
        texture: Texture::new(ctx, "./examples/resources/player.png")?,
    };

    tetra::run(ctx, state)
}
