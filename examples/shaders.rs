use tetra::graphics::{self, Color, DrawParams, Shader, Texture, Vec2};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    shader: Shader,
    red: f32,
    green: f32,
    blue: f32,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
            shader: Shader::fragment(ctx, "./examples/resources/disco.frag")?,
            red: 0.0,
            green: 0.0,
            blue: 0.0,
        })
    }
}

impl State for GameState {
    fn update(&mut self, _ctx: &mut Context) -> tetra::Result {
        self.red += 0.1;
        self.green += 0.05;
        self.blue += 0.025;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));
        graphics::set_shader(ctx, &self.shader);

        self.shader
            .set_uniform(ctx, "u_red", (self.red.sin() + 1.0) / 2.0);

        self.shader
            .set_uniform(ctx, "u_green", (self.green.sin() + 1.0) / 2.0);

        self.shader
            .set_uniform(ctx, "u_blue", (self.blue.sin() + 1.0) / 2.0);

        graphics::draw(
            ctx,
            &self.texture,
            DrawParams::new()
                .position(Vec2::new(80.0, 72.0))
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(4.0, 4.0)),
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
