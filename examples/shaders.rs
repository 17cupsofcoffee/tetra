use tetra::graphics::{self, Color, Texture, Vec2, Shader};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    shader: Shader,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
            shader: Shader::new(ctx, "./src/resources/shader.vert", "./examples/resources/greyscale.frag")?
        })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));
        graphics::set_shader(ctx, &self.shader);
        graphics::draw(ctx, &self.texture, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Rendering a Texture", 160, 144)
        .maximized(true)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run_with(GameState::new)
}
