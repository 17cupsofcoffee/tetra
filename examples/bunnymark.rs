/// Based on https://github.com/openfl/openfl-samples/tree/master/demos/BunnyMark
/// Original BunnyMark (and sprite) by Iain Lobb
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use tetra::graphics::{self, Color, Texture};
use tetra::input::{self, Key, MouseButton};
use tetra::math::Vec2;
use tetra::time;
use tetra::window;
use tetra::{Context, ContextBuilder, State};

// NOTE: Using a high number here yields worse performance than adding more bunnies over
// time - I think this is due to all of the RNG being run on the same tick...
const INITIAL_BUNNIES: usize = 100;
const MAX_X: f32 = 1280.0 - 26.0;
const MAX_Y: f32 = 720.0 - 37.0;
const GRAVITY: f32 = 0.5;

struct Bunny {
    position: Vec2<f32>,
    velocity: Vec2<f32>,
}

impl Bunny {
    fn new(rng: &mut ThreadRng) -> Bunny {
        let x_vel = rng.gen::<f32>() * 5.0;
        let y_vel = (rng.gen::<f32>() * 5.0) - 2.5;

        Bunny {
            position: Vec2::new(0.0, 0.0),
            velocity: Vec2::new(x_vel, y_vel),
        }
    }
}

struct GameState {
    rng: ThreadRng,
    texture: Texture,
    bunnies: Vec<Bunny>,

    auto_spawn: bool,
    spawn_timer: i32,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut rng = rand::thread_rng();
        let texture = Texture::new(ctx, "./examples/resources/wabbit_alpha.png")?;
        let mut bunnies = Vec::with_capacity(INITIAL_BUNNIES);

        for _ in 0..INITIAL_BUNNIES {
            bunnies.push(Bunny::new(&mut rng));
        }

        Ok(GameState {
            rng,
            texture,
            bunnies,

            auto_spawn: false,
            spawn_timer: 0,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.spawn_timer > 0 {
            self.spawn_timer -= 1;
        }

        if input::is_key_pressed(ctx, Key::A) {
            self.auto_spawn = !self.auto_spawn;
        }

        let should_spawn = self.spawn_timer == 0
            && (input::is_mouse_button_down(ctx, MouseButton::Left) || self.auto_spawn);

        if should_spawn {
            for _ in 0..INITIAL_BUNNIES {
                self.bunnies.push(Bunny::new(&mut self.rng));
            }
            self.spawn_timer = 10;
        }

        for bunny in &mut self.bunnies {
            bunny.position += bunny.velocity;
            bunny.velocity.y += GRAVITY;

            if bunny.position.x > MAX_X {
                bunny.velocity.x *= -1.0;
                bunny.position.x = MAX_X;
            } else if bunny.position.x < 0.0 {
                bunny.velocity.x *= -1.0;
                bunny.position.x = 0.0;
            }

            if bunny.position.y > MAX_Y {
                bunny.velocity.y *= -0.8;
                bunny.position.y = MAX_Y;

                if self.rng.gen::<bool>() {
                    bunny.velocity.y -= 3.0 + (self.rng.gen::<f32>() * 4.0);
                }
            } else if bunny.position.y < 0.0 {
                bunny.velocity.y = 0.0;
                bunny.position.y = 0.0;
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        for bunny in &self.bunnies {
            self.texture.draw(ctx, bunny.position);
        }

        window::set_title(
            ctx,
            &format!(
                "BunnyMark - {} bunnies - {:.0} FPS",
                self.bunnies.len(),
                time::get_fps(ctx)
            ),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("BunnyMark", 1280, 720)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
