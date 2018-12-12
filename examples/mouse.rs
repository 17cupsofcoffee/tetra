use tetra::glm::{self, Vec2};
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::input::{self, MouseButton};
use tetra::{self, Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    position: Vec2,
    scale: Vec2,
    rotation: f32,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) {
        self.position = glm::round(&input::get_mouse_position(ctx));

        if input::is_mouse_button_down(ctx, MouseButton::Left) {
            self.scale = Vec2::new(2.0, 2.0);
            self.rotation += 0.1;
        } else {
            self.scale = Vec2::new(1.0, 1.0);
            self.rotation = 0.0;
        }
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::draw(
            ctx,
            &self.texture,
            DrawParams::new()
                .position(self.position)
                .origin(Vec2::new(8.0, 8.0))
                .scale(self.scale)
                .rotation(self.rotation),
        );
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Mouse Input")
        .size(160, 144)
        .scale(4)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState {
        texture: Texture::new(ctx, "./examples/resources/player.png")?,
        position: Vec2::new(160.0 / 2.0, 144.0 / 2.0),
        scale: Vec2::new(1.0, 1.0),
        rotation: 0.0,
    };

    tetra::run(ctx, state)
}
