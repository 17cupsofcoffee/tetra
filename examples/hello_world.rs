use tetra::graphics::{self, Color};
use tetra::{Context, ContextBuilder, State};

struct GameState;

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        // Cornflour blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        Ok(())
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new("Hello, world!", 1280, 720).build()?;
    let state = &mut GameState;

    tetra::run(ctx, state)
}
