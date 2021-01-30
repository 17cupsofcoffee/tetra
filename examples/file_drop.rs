use std::fs;

use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State, TetraError};

struct GameState {
    file: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            file: Text::new(
                "Drop a file onto this window to view the contents.",
                Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 16.0)?,
            ),
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.file.draw(ctx, Vec2::new(16.0, 16.0));

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::FileDropped { path } = event {
            let new_content = fs::read_to_string(&path)
                .map_err(|e| TetraError::FailedToLoadAsset { reason: e, path })?;

            self.file.set_content(new_content);
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("File Dropping", 1280, 720)
        .build()?
        .run(GameState::new)
}
