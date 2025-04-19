//! This example demonstrates how an ECS library can be integrated into Tetra,
//! by porting the logic from the 'bunnymark' example into systems.
//!
//! The below code uses Hecs, but other libraries (such as Specs)
//! should work too.

use hecs::World;
use rand::rngs::StdRng;
use rand::{self, Rng, SeedableRng};

use tetra::graphics::{self, Color, Texture};
use tetra::input::{self, Key, MouseButton};
use tetra::math::Vec2;
use tetra::time;
use tetra::window;
use tetra::{Context, ContextBuilder, State};

const INITIAL_BUNNIES: usize = 100;
const MAX_X: f32 = 1280.0 - 26.0;
const MAX_Y: f32 = 720.0 - 37.0;
const GRAVITY: f32 = 0.5;

struct Position(Vec2<f32>);

struct Velocity(Vec2<f32>);

struct Input {
    clicked: bool,
    auto_spawn: bool,
}

struct Resources {
    input: Input,
    spawn_timer: i32,
    rng: StdRng,
}

fn create_bunny(rng: &mut StdRng) -> (Position, Velocity) {
    let x_vel = rng.random::<f32>() * 5.0;
    let y_vel = (rng.random::<f32>() * 5.0) - 2.5;

    (Position(Vec2::zero()), Velocity(Vec2::new(x_vel, y_vel)))
}

struct GameState {
    world: World,
    resources: Resources,
    texture: Texture,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<Self> {
        let mut rng = StdRng::from_os_rng();

        let mut world = World::default();

        for _ in 0..INITIAL_BUNNIES {
            world.spawn_batch((0..INITIAL_BUNNIES).map(|_| create_bunny(&mut rng)));
        }

        let input = Input {
            clicked: false,
            auto_spawn: false,
        };

        let resources = Resources {
            input,
            rng,
            spawn_timer: 0,
        };

        Ok(GameState {
            world,
            resources,
            texture: Texture::new(ctx, "./examples/resources/wabbit_alpha.png")?,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        read_input(ctx, &mut self.resources);
        spawn_bunnies(&mut self.world, &mut self.resources);
        update_positions(&mut self.world, &mut self.resources);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        render_world(ctx, &self.world, &self.texture);

        window::set_title(
            ctx,
            format!(
                "ECS BunnyMark - {} bunnies - {:.0} FPS",
                self.world.len(),
                time::get_fps(ctx)
            ),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("ECS BunnyMark", 1280, 720)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}

fn read_input(ctx: &mut Context, res: &mut Resources) {
    res.input.clicked = input::is_mouse_button_down(ctx, MouseButton::Left);

    if input::is_key_pressed(ctx, Key::A) {
        res.input.auto_spawn = !res.input.auto_spawn;
    }
}

fn spawn_bunnies(world: &mut World, res: &mut Resources) {
    if res.spawn_timer > 0 {
        res.spawn_timer -= 1;
    }

    if res.spawn_timer == 0 && (res.input.clicked || res.input.auto_spawn) {
        world.spawn_batch((0..INITIAL_BUNNIES).map(|_| create_bunny(&mut res.rng)));

        res.spawn_timer = 10;
    }
}

fn update_positions(world: &mut World, res: &mut Resources) {
    for (_, (Position(position), Velocity(velocity))) in
        world.query_mut::<(&mut Position, &mut Velocity)>()
    {
        *position += *velocity;
        velocity.y += GRAVITY;

        if position.x > MAX_X {
            velocity.x *= -1.0;
            position.x = MAX_X;
        } else if position.x < 0.0 {
            velocity.x *= -1.0;
            position.x = 0.0;
        }

        if position.y > MAX_Y {
            velocity.y *= -0.8;
            position.y = MAX_Y;

            if res.rng.random::<bool>() {
                velocity.y -= 3.0 + (res.rng.random::<f32>() * 4.0);
            }
        } else if position.y < 0.0 {
            velocity.y = 0.0;
            position.y = 0.0;
        }
    }
}

fn render_world(ctx: &mut Context, world: &World, texture: &Texture) {
    graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

    for (_, position) in &mut world.query::<&Position>() {
        texture.draw(ctx, position.0);
    }
}
