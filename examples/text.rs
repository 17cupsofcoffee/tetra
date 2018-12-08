extern crate tetra;

use tetra::glm::Vec2;
use tetra::graphics::{self, Color, Font, Text};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    text: Text,
    pos: Vec2,
}

impl State for GameState {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, self.pos);
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Rendering text")
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

    tetra::run(ctx, state)
}
