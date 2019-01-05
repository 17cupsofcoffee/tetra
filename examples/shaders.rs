use tetra::graphics::{self, Color, DrawParams, Shader, Texture, Vec2};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    shader: Shader,
    timer: i32,
    enabled: bool,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
            shader: Shader::fragment(ctx, "./examples/resources/disco.frag")?,
            timer: 0,
            enabled: false,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.timer += 1;

        if self.timer % 60 == 0 {
            self.enabled = !self.enabled;
            self.timer = 0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        if self.enabled {
            graphics::set_shader(ctx, &self.shader);
        } else {
            graphics::reset_shader(ctx);
        }

        graphics::draw(
            ctx,
            &self.texture,
            DrawParams::new()
                .position(Vec2::new(80.0, 72.0))
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(4.0, 4.0))
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Custom Shaders", 160, 144)
        .maximized(true)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run_with(GameState::new)
}
