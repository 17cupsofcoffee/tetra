use tetra::graphics::{self, Color, DrawParams, Vec2, Texture, Animation, Rectangle};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    animation: Animation,
    position: Vec2,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            animation: Animation::new(
                Texture::new(ctx, "./examples/resources/tiles.png")?,
                Rectangle::row(0.0, 256.0, 16.0, 16.0).take(8).collect(),
                5,
            ),
            position: Vec2::new(240.0, 160.0),
        })
    }

    pub fn player_idle(&mut self) {
        self.animation.set_frames(Rectangle::row(0.0, 256.0, 16.0, 16.0).take(8).collect());
    }

    pub fn player_walk(&mut self) {
        self.animation.set_frames(Rectangle::row(0.0, 272.0, 16.0, 16.0).take(8).collect());
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.animation.tick();

        if input::is_key_down(ctx, Key::A) {
            self.position.x -= 1.5;
        }

        if input::is_key_pressed(ctx, Key::A) {
            self.player_walk();
        }

        if input::is_key_released(ctx, Key::A) {
            self.player_idle();
        }

        if input::is_key_down(ctx, Key::D) {
            self.position.x += 1.5;
        }

        if input::is_key_pressed(ctx, Key::D) {
            self.player_walk();
        }

        if input::is_key_released(ctx, Key::D) {
            self.player_idle();
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::draw(
            ctx,
            &self.animation,
            DrawParams::new()
                .position(self.position)
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(2.0, 2.0)),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Keyboard Input with animation", 480, 320)
        .maximized(false)
        .resizable(false)
        .quit_on_escape(true)
        .build()?
        .run_with(GameState::new)
}
