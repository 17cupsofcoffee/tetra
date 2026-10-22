use tetra::graphics::text::{Font, HorizontalAlignment, Text, TextAlignment, VerticalAlignment};
use tetra::graphics::{self, Color};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

const TEXT_OFFSET: Vec2<f32> = Vec2::new(16.0, 16.0);

struct GameState {
    vector_text: Text,
    bitmap_text: Text,
    centered_text: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let vector_font = Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 16.0)?;

        let vector_text = Text::new(
            "Hello, world!\n\nThis is some text being rendered from a TTF font.",
            vector_font.clone(),
        );

        let centered_text = Text::new("This line is centered.", vector_font);

        let bitmap_text = Text::new(
            "Hello, world!\n\nThis is some text being rendered from a bitmap font.",
            Font::bmfont(ctx, "./examples/resources/DejaVuSansMono.fnt")?,
        );

        Ok(GameState {
            vector_text,
            bitmap_text,
            centered_text,
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.vector_text.draw(ctx, TEXT_OFFSET);
        self.bitmap_text
            .draw(ctx, TEXT_OFFSET + Vec2::new(0.0, 128.0));
        self.centered_text.draw_aligned(
            ctx,
            Vec2::new(640.0, 360.0),
            TextAlignment::new(HorizontalAlignment::Center, VerticalAlignment::Middle),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Rendering Text", 1280, 720)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
