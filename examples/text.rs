use tetra::graphics::{self, Color, Font, Text, Vec2};
use tetra::{self, Context, ContextBuilder, State};

struct GameState {
    text: Text,
    pos: Vec2,
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, self.pos);

        Ok(())
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new("Rendering text", 1280, 720)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState {
        text: Text::new(
            "Hello, world!\n\nThis is some text being rendered from a TTF font.",
            Font::default(),
            16.0,
        ),
        pos: Vec2::new(16.0, 16.0),
    };

    println!("Text bounds are {:?}", state.text.get_bounds(ctx));

    tetra::run(ctx, state)
}
