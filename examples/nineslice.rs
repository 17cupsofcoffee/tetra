use tetra::graphics::{self, Color, NineSlice, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    config: NineSlice,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let texture = Texture::new(ctx, "./examples/resources/panel.png")?;

        Ok(GameState {
            texture,
            config: NineSlice::with_border(Rectangle::new(0.0, 0.0, 32.0, 32.0), 4.0),
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);

        self.texture
            .draw_nine_slice(ctx, &self.config, 640.0, 480.0, Vec2::zero());

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Rendering a NineSlice", 640, 480)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
