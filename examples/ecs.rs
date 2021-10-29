//! This example demonstrates how an ECS library can be integrated into Tetra,
//! by porting the logic from the 'bunnymark' example into systems.
//!
//! The below code uses Legion, but other libraries (such as Specs and Hecs)
//! should work too.

use legion::systems::CommandBuffer;
use legion::*;
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

struct SpawnTimer(i32);

fn create_bunny(rng: &mut StdRng) -> (Position, Velocity) {
    let x_vel = rng.gen::<f32>() * 5.0;
    let y_vel = (rng.gen::<f32>() * 5.0) - 2.5;

    (Position(Vec2::zero()), Velocity(Vec2::new(x_vel, y_vel)))
}

#[system]
fn spawn_bunnies(
    cmd: &mut CommandBuffer,
    #[resource] input: &Input,
    #[resource] SpawnTimer(spawn_timer): &mut SpawnTimer,
    #[resource] rng: &mut StdRng,
) {
    if *spawn_timer > 0 {
        *spawn_timer -= 1;
    }

    if *spawn_timer == 0 && (input.clicked || input.auto_spawn) {
        for _ in 0..INITIAL_BUNNIES {
            cmd.push(create_bunny(rng));
        }

        *spawn_timer = 10;
    }
}

#[system(for_each)]
fn update_positions(
    Position(position): &mut Position,
    Velocity(velocity): &mut Velocity,
    #[resource] rng: &mut StdRng,
) {
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

        if rng.gen::<bool>() {
            velocity.y -= 3.0 + (rng.gen::<f32>() * 4.0);
        }
    } else if position.y < 0.0 {
        velocity.y = 0.0;
        position.y = 0.0;
    }
}

struct GameState {
    world: World,
    schedule: Schedule,
    resources: Resources,
    texture: Texture,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<Self> {
        let mut rng = StdRng::from_entropy();

        let mut world = World::default();

        for _ in 0..INITIAL_BUNNIES {
            world.push(create_bunny(&mut rng));
        }

        let mut resources = Resources::default();

        resources.insert(rng);
        resources.insert(Input {
            clicked: false,
            auto_spawn: false,
        });
        resources.insert(SpawnTimer(0));

        Ok(GameState {
            world,
            resources,
            schedule: Schedule::builder()
                .add_system(spawn_bunnies_system())
                .add_system(update_positions_system())
                .build(),

            texture: Texture::new(ctx, "./examples/resources/wabbit_alpha.png")?,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        // A common issue people run into when trying to use ECS libraries (and multi-threading
        // in general) with Tetra is how to handle the fact that the Context cannot be
        // accessed from other threads.
        //
        // One way to deal with this is to decouple your game logic from the raw input. In this
        // example, we store the state of the player's input into a resource, and then read
        // that in the systems. This has the additional benefit of making it very easy to
        // change input bindings/add input configuration later on!
        //
        // Similarly, if you want to access things like the delta time inside a system, write
        // them into a resource before running your system scheduler.

        {
            let mut input = self.resources.get_mut::<Input>().unwrap();
            input.clicked = input::is_mouse_button_down(ctx, MouseButton::Left);

            if input::is_key_pressed(ctx, Key::A) {
                input.auto_spawn = !input.auto_spawn;
            }
        }

        self.schedule.execute(&mut self.world, &mut self.resources);

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        // Similar to the above issue with input, rendering can also be a bit tricky
        // when using an ECS library, due to the relevant types not being thread-safe.
        //
        // The easiest way to work around this is to just render outside of the ECS
        // systems, where you have full access to the Context.

        let mut bunnies = <&Position>::query();

        for position in bunnies.iter(&self.world) {
            self.texture.draw(ctx, position.0);
        }

        window::set_title(
            ctx,
            &format!(
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
