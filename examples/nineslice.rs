use tetra::glm::Vec2;
use tetra::graphics::color;
use tetra::graphics::{self, NineSlice, Rectangle, Texture};
use tetra::{self, Context, ContextBuilder, State};

struct GameState {
    panel: NineSlice,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let texture = Texture::new(ctx, "./examples/resources/panel.png")?;

        Ok(GameState {
            panel: NineSlice::new(texture, 160.0, 144.0, Rectangle::new(4.0, 4.0, 24.0, 24.0)),
        })
    }
}

impl State for GameState {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, color::BLACK);

        graphics::draw(ctx, &self.panel, Vec2::new(0.0, 0.0));
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Rendering a NineSlice")
        .size(160, 144)
        .window_scale(4)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState::new(ctx)?;

    tetra::run(ctx, state)
}
