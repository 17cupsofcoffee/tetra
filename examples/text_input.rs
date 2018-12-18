use tetra::graphics::{self, Color, Font, Text, Vec2};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    input: String,
    text: Text,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_pressed(ctx, Key::Return) {
            self.input += "\n";
            self.text.set_content(self.input.as_str());
        }

        if input::is_key_pressed(ctx, Key::Backspace) {
            self.input.pop();
            self.text.set_content(self.input.as_str());
        }

        if let Some(new_input) = input::get_text_input(ctx) {
            self.input += new_input;
            self.text.set_content(self.input.as_str());
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.text, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new("Keyboard Input", 640, 480)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState {
        input: String::new(),
        text: Text::new("", Font::default(), 32.0),
    };

    tetra::run(ctx, state)
}
