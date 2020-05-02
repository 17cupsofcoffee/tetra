use tetra::graphics::{self, Color, Font, Text};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};

struct GameState {
    text: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let text = Text::new(
            "Look at your console to see what events are being fired!",
            Font::new(ctx, "./examples/resources/DejaVuSansMono.ttf")?,
            16.0,
        );

        Ok(GameState { text })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, Vec2::new(16.0, 16.0));

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        println!("{:?}", event);
        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Events", 1280, 720)
        .resizable(true)
        .build()?
        .run(GameState::new)
}
