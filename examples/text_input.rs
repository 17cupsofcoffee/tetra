use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color};
use tetra::input::{self, Key, KeyModifier};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    text: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let font = Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 32.0)?;

        Ok(GameState {
            text: Text::new("", font),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_pressed(ctx, Key::Enter) {
            self.text.push_str("\n");
        }

        if input::is_key_pressed(ctx, Key::Backspace) {
            self.text.pop();
        }

        if let Some(new_input) = input::get_text_input(ctx) {
            self.text.push_str(new_input);
        }

        if input::is_key_modifier_down(ctx, KeyModifier::Ctrl) {
            if input::is_key_pressed(ctx, Key::C) {
                input::set_clipboard_text(ctx, self.text.content())?;
            }

            if input::is_key_pressed(ctx, Key::V) {
                self.text.push_str(&input::get_clipboard_text(ctx)?);
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.text.draw(ctx, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Keyboard Input", 640, 480)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
