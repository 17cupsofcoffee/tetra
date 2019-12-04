use tetra::graphics::{self, Camera, Color, DrawParams, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};

const CAMERA_SPEED: f32 = 4.0;

struct GameState {
    texture: Texture,
    camera: Camera,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/player.png")?,
            camera: Camera::with_window_size(ctx),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_down(ctx, Key::W) {
            self.camera.position.y -= CAMERA_SPEED;
        }

        if input::is_key_down(ctx, Key::S) {
            self.camera.position.y += CAMERA_SPEED;
        }

        if input::is_key_down(ctx, Key::A) {
            self.camera.position.x -= CAMERA_SPEED;
        }

        if input::is_key_down(ctx, Key::D) {
            self.camera.position.x += CAMERA_SPEED;
        }

        if input::is_key_down(ctx, Key::Q) {
            self.camera.rotation -= 0.1;
        }

        if input::is_key_down(ctx, Key::E) {
            self.camera.rotation += 0.1;
        }

        if input::is_key_down(ctx, Key::R) {
            self.camera.zoom += 0.1;
        }

        if input::is_key_down(ctx, Key::F) {
            self.camera.zoom -= 0.1;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.769, 0.812, 0.631));

        graphics::set_transform_matrix(ctx, self.camera.to_matrix());

        graphics::draw(
            ctx,
            &self.texture,
            DrawParams::new()
                .origin(Vec2::new(8.0, 8.0))
                .scale(Vec2::new(2.0, 2.0)),
        );

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.camera.set_viewport_size(width as f32, height as f32);
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Cameras", 640, 480)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
