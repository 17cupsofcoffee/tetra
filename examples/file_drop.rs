use std::fs;

use tetra::graphics::{self, Color, Font, Text};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};

struct GameState {
    file: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            file: Text::new(
                "Drop a file onto this window to view the contents.",
                Font::new(ctx, "./examples/resources/DejaVuSansMono.ttf")?,
                16.0,
            ),
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.file, Vec2::new(16.0, 16.0));

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::FileDropped { path } = event {
            let new_content = fs::read_to_string(path).unwrap();

            *self.file.content_mut() = new_content;
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("File Dropping", 1280, 720)
        .build()?
        .run(GameState::new)
}
