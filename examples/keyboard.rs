extern crate tetra;

use tetra::error::Result;
use tetra::glm::Vec2;
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

struct GameState {
    texture: Texture,
    position: Vec2,
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) {
        if input::is_key_down(ctx, Key::A) {
            self.position.x -= 2.0;
        }

        if input::is_key_down(ctx, Key::D) {
            self.position.x += 2.0;
        }

        if input::is_key_down(ctx, Key::W) {
            self.position.y -= 2.0;
        }

        if input::is_key_down(ctx, Key::S) {
            self.position.y += 2.0;
        }
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::draw(
            ctx,
            &self.texture,
            DrawParams::new()
                .position(self.position)
                .origin(Vec2::new(8.0, 8.0)),
        );
    }
}

fn main() -> Result {
    let ctx = &mut ContextBuilder::new()
        .title("Rendering a Texture")
        .size(160, 144)
        .scale(4)
        .resizable(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState {
        texture: Texture::new(ctx, "./examples/resources/player.png")?,
        position: Vec2::new(160.0 / 2.0, 144.0 / 2.0),
    };

    tetra::run(ctx, state)
}
