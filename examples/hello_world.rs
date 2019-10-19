use tetra::graphics::{self, Color};
use tetra::{Context, Settings, State};

struct GameState;

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        Ok(())
    }
}

fn main() {
    tetra::run(&Settings::new("Hello, world!", 1280, 720), |_| {
        Ok(GameState)
    });
}
