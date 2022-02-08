use tetra::graphics::{self, Color};
use tetra::{Context, ContextBuilder, State};

struct GameState;

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        // Cornflower blue, as is tradition
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        Ok(())
    }

    #[cfg(feature = "experimental_imgui")]
    fn draw_imgui(&mut self, ui: &mut tetra::imgui::Ui) -> Result<(), tetra::TetraError> {
        ui.show_demo_window(&mut true);
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Hello, world!", 1280, 720)
        .show_mouse(true)
        .build()?
        .run(|_| Ok(GameState))
}
