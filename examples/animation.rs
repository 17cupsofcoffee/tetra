// using sprites by 0x72: https://0x72.itch.io/16x16-industrial-tileset

use tetra::glm::Vec2;
use tetra::graphics::{self, Animation, Color, DrawParams, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    animation: Animation,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            animation: Animation::new(
                Texture::new(ctx, "./examples/resources/tiles.png")?,
                Rectangle::row(0.0, 272.0, 16.0, 16.0).take(8).collect(),
                5,
            ),
        })
    }

    pub fn set_animation_1(&mut self) {
        self.animation
            .set_frames(Rectangle::row(0.0, 272.0, 16.0, 16.0).take(8).collect());
    }

    pub fn set_animation_2(&mut self) {
        self.animation
            .set_frames(Rectangle::row(0.0, 256.0, 16.0, 16.0).take(8).collect());
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) {
        self.animation.tick();
        if input::is_key_pressed(ctx, Key::Num1) {
            self.set_animation_1();
        }
        if input::is_key_pressed(ctx, Key::Num2) {
            self.set_animation_2();
        }
        if input::is_key_pressed(ctx, Key::Space) {
            self.animation.restart();
        }
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(
            ctx,
            &self.animation,
            DrawParams::new()
                .position(Vec2::new(32.0, 32.0))
                .origin(Vec2::new(8.0, 8.0)),
        );
    }
}

fn main() -> tetra::Result {
    let ctx = &mut ContextBuilder::new()
        .title("Displaying an Animation")
        .size(64, 64)
        .window_scale(8)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState::new(ctx)?;

    tetra::run(ctx, state)
}
