/// Based on https://github.com/openfl/openfl-samples/tree/master/demos/BunnyMark
/// Original BunnyMark (and sprite) by Iain Lobb
use rand::rngs::ThreadRng;
use rand::{self, Rng};
use tetra::graphics::{self, Color, Texture, Vec2};
use tetra::input::{self, MouseButton};
use tetra::time;
use tetra::window;
use tetra::{Context, ContextBuilder, State};

// NOTE: Using a high number here yields worse performance than adding more bunnies over
// time - I think this is due to all of the RNG being run on the same tick...
const INITIAL_BUNNIES: usize = 100;
const WIDTH: i32 = 1280;
const HEIGHT: i32 = 720;
const GRAVITY: f32 = 0.5;

struct Bunny {
    position: Vec2,
    velocity: Vec2,
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
    max_x: f32,
    max_y: f32,

    click_timer: i32,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let mut rng = rand::thread_rng();
        let texture = Texture::new(ctx, "./examples/resources/wabbit_alpha.png")?;
        let mut bunnies = Vec::with_capacity(INITIAL_BUNNIES);
        let max_x = (WIDTH - texture.width()) as f32;
        let max_y = (HEIGHT - texture.height()) as f32;

        for _ in 0..INITIAL_BUNNIES {
            bunnies.push(Bunny::new(&mut rng));
        }

        Ok(GameState {
            rng,
            texture,
            bunnies,
            max_x,
            max_y,

            click_timer: 0,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if self.click_timer > 0 {
            self.click_timer -= 1;
        }

        if input::is_mouse_button_down(ctx, MouseButton::Left) && self.click_timer == 0 {
            for _ in 0..INITIAL_BUNNIES {
                self.bunnies.push(Bunny::new(&mut self.rng));
            }
            self.click_timer = 10;
        }

        for bunny in &mut self.bunnies {
            bunny.position += bunny.velocity;
            bunny.velocity.y += GRAVITY;

            if bunny.position.x > self.max_x {
                bunny.velocity.x *= -1.0;
                bunny.position.x = self.max_x;
            } else if bunny.position.x < 0.0 {
                bunny.velocity.x *= -1.0;
                bunny.position.x = 0.0;
            }

            if bunny.position.y > self.max_y {
                bunny.velocity.y *= -0.8;
                bunny.position.y = self.max_y;

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

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        for bunny in &self.bunnies {
            graphics::draw(ctx, &self.texture, bunny.position);
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
    ContextBuilder::new("BunnyMark", WIDTH, HEIGHT)
        .quit_on_escape(true)
        .build()?
        .run_with(GameState::new)
}
