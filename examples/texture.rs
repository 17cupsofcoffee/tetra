use tetra::graphics::{self, Color, DrawParams, ImageData, Texture};
use tetra::math::Vec2;
use tetra::{window, Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut image = ImageData::from_file("./examples/resources/player.png")?;

        window::set_icon(ctx, &mut image)?;

        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        self.texture.draw(
            ctx,
            DrawParams::new()
                .position(Vec2::new(32.0, 32.0))
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(2.0, 2.0)),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Rendering a Texture", 640, 480)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
