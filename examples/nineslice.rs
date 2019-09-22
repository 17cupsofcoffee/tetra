use tetra::graphics::ui::NineSlice;
use tetra::graphics::{self, Color, Rectangle, Texture};
use tetra::math::Vec2;
use tetra::{Context, Game, State};

struct GameState {
    panel: NineSlice,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let texture = Texture::new(ctx, "./examples/resources/panel.png")?;

        Ok(GameState {
            panel: NineSlice::new(
                texture,
                640.0 - 32.0,
                480.0 - 32.0,
                Rectangle::new(4.0, 4.0, 24.0, 24.0),
            ),
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::BLACK);
        graphics::draw(ctx, &self.panel, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() {
    Game::new("Rendering a NineSlice", 640, 480)
        .quit_on_escape(true)
        .run(GameState::new);
}
