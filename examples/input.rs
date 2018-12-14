extern crate tetra;

use tetra::glm::Vec2;
use tetra::graphics::{self, Color};
use tetra::graphics::text::{Text, Font};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    input: String,
    text: Text,
}

impl GameState {
    fn update_text(&mut self) {
        self.text = Text::new(
            self.input.as_str(),
            Font::default(),
            16.0,
        );
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) {
        if input::is_key_pressed(ctx, Key::Backspace) {
            self.input.pop();
            self.update_text();
        }
        if let Some(new_input) = input::get_text_input(ctx) {
            self.input += new_input;
            self.update_text();
        }
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::draw(ctx, &self.text, Vec2::new(50., 50.));
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Keyboard Input")
        .size(320, 132)
        .scale(4)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState {
        input: String::new(),
        text: Text::new(
            "",
            Font::default(),
            16.0,
        ),
    };

    tetra::run(ctx, state)
}
