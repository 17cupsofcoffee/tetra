use tetra::graphics::{self, Color, Font, Text};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    text: Text,
    pos: Vec2<f32>,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let text = Text::new(
            "Hello, world!\n\nThis is some text being rendered from a TTF font.",
            Font::new(ctx, "./examples/resources/DejaVuSansMono.ttf")?,
            16.0,
        );

        println!("Text bounds are {:?}", text.get_bounds(ctx));

        Ok(GameState {
            text,
            pos: Vec2::new(16.0, 16.0),
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, self.pos);

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Rendering Text", 1280, 720)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
