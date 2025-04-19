// This example demonstrates how you might go about reusing a `Font` instance for more performant
// text rendering. `Font` instances are expensive to create but cheap to `.clone()`.
//
// In your game, you could have a handful of different `Font` instances for different size text or
// different typefaces. You'd then draw your text accordingly based on which font is appropriate,
// `.clone`-ing those instances.
//
// Press SPACE to create more `Text` instances.

use rand::distr::{Distribution, Uniform};
use rand::rngs::ThreadRng;
use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

const TEXT_OFFSET: Vec2<f32> = Vec2::new(16.0, 16.0);
const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;

struct GameState {
    texts: Vec<(Text, f32, f32)>,
    font: Font,
    x_between: Uniform<i32>,
    y_between: Uniform<i32>,
    rng: ThreadRng,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut state = GameState {
            texts: vec![],
            font: Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 16.0)?,
            rng: rand::rng(),
            x_between: Uniform::try_from(0..WIDTH).unwrap(),
            y_between: Uniform::try_from(0..HEIGHT).unwrap(),
        };

        state.add_texts();

        Ok(state)
    }

    fn add_texts(&mut self) {
        for i in 1..=200 {
            self.texts.push((
                // NOTE: rather than making a new font, we clone the already existing one. This is
                // much more performant.
                Text::new(format!("text {}", i), self.font.clone()),
                self.x_between.sample(&mut self.rng) as f32,
                self.y_between.sample(&mut self.rng) as f32,
            ));
        }
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        tetra::window::set_title(
            ctx,
            format!(
                "Text Perf ({} texts, {:.0} FPS)",
                self.texts.len(),
                tetra::time::get_fps(ctx)
            ),
        );

        if tetra::input::is_key_pressed(ctx, tetra::input::Key::Space) {
            self.add_texts();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        for (text, x, y) in &mut self.texts {
            text.draw(ctx, TEXT_OFFSET + Vec2::new(*x, *y));
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Text Perf", WIDTH, HEIGHT)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
