// Button prompts from https://opengameart.org/content/free-keyboard-and-controllers-prompts-pack

use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color, DrawParams, Rectangle, Texture};
use tetra::input::{self, GamepadAxis, GamepadButton, GamepadStick};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

enum Sprite {
    A,
    B,
    X,
    Y,
    Up,
    Down,
    Left,
    Right,
    LeftBumper,
    LeftTrigger,
    RightBumper,
    RightTrigger,
    LeftStick,
    RightStick,
    Start,
    Back,
    Disconnected,
}

impl From<Sprite> for Rectangle {
    fn from(sprite: Sprite) -> Rectangle {
        let (u, v) = match sprite {
            Sprite::A => (0.0, 0.0),
            Sprite::B => (100.0, 0.0),
            Sprite::X => (200.0, 0.0),
            Sprite::Y => (300.0, 0.0),
            Sprite::Up => (0.0, 100.0),
            Sprite::Down => (100.0, 100.0),
            Sprite::Left => (200.0, 100.0),
            Sprite::Right => (300.0, 100.0),
            Sprite::LeftBumper => (0.0, 200.0),
            Sprite::LeftTrigger => (100.0, 200.0),
            Sprite::RightBumper => (200.0, 200.0),
            Sprite::RightTrigger => (300.0, 200.0),
            Sprite::LeftStick => (0.0, 300.0),
            Sprite::RightStick => (100.0, 300.0),
            Sprite::Start => (200.0, 300.0),
            Sprite::Back => (300.0, 300.0),
            Sprite::Disconnected => (0.0, 400.0),
        };

        Rectangle::new(u, v, 100.0, 100.0)
    }
}

struct GameState {
    texture: Texture,
    active_color: Color,

    connected: bool,

    a: bool,
    b: bool,
    x: bool,
    y: bool,

    up: bool,
    down: bool,
    left: bool,
    right: bool,

    lb: bool,
    lt: bool,
    rb: bool,
    rt: bool,

    start: bool,
    back: bool,

    left_stick: Vec2<f32>,
    right_stick: Vec2<f32>,

    axis_info: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            texture: Texture::new(ctx, "./examples/resources/controls.png")?,
            active_color: Color::rgb(1.0, 0.5, 0.5),

            connected: true,

            a: false,
            b: false,
            x: false,
            y: false,

            up: false,
            down: false,
            left: false,
            right: false,

            lb: false,
            lt: false,
            rb: false,
            rt: false,

            start: false,
            back: false,

            left_stick: Vec2::zero(),
            right_stick: Vec2::zero(),

            axis_info: Text::new(
                "",
                Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 16.0)?,
            ),
        })
    }

    fn draw_button(&self, ctx: &mut Context, x: f32, y: f32, sprite: Sprite, active: bool) {
        self.texture.draw_region(
            ctx,
            sprite.into(),
            DrawParams::new()
                .position(Vec2::new(x, y))
                .color(if active {
                    self.active_color
                } else {
                    Color::WHITE
                }),
        );
    }

    fn draw_stick(&self, ctx: &mut Context, x: f32, y: f32, sprite: Sprite, value: Vec2<f32>) {
        self.texture.draw_region(
            ctx,
            sprite.into(),
            DrawParams::new()
                .position(Vec2::new(x, y) + (value * 32.0))
                .color(if value.magnitude().abs() > 0.08 {
                    self.active_color
                } else {
                    Color::WHITE
                }),
        );
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.connected = input::is_gamepad_connected(ctx, 0);

        if self.connected {
            if input::get_gamepad_buttons_pressed(ctx, 0).count() > 0 {
                input::start_gamepad_vibration(ctx, 0, 1.0, 100);
            }

            self.a = input::is_gamepad_button_down(ctx, 0, GamepadButton::A);
            self.b = input::is_gamepad_button_down(ctx, 0, GamepadButton::B);
            self.x = input::is_gamepad_button_down(ctx, 0, GamepadButton::X);
            self.y = input::is_gamepad_button_down(ctx, 0, GamepadButton::Y);

            self.up = input::is_gamepad_button_down(ctx, 0, GamepadButton::Up);
            self.down = input::is_gamepad_button_down(ctx, 0, GamepadButton::Down);
            self.left = input::is_gamepad_button_down(ctx, 0, GamepadButton::Left);
            self.right = input::is_gamepad_button_down(ctx, 0, GamepadButton::Right);

            self.lb = input::is_gamepad_button_down(ctx, 0, GamepadButton::LeftShoulder);
            self.lt = input::is_gamepad_button_down(ctx, 0, GamepadButton::LeftTrigger);
            self.rb = input::is_gamepad_button_down(ctx, 0, GamepadButton::RightShoulder);
            self.rt = input::is_gamepad_button_down(ctx, 0, GamepadButton::RightTrigger);

            self.start = input::is_gamepad_button_down(ctx, 0, GamepadButton::Start);
            self.back = input::is_gamepad_button_down(ctx, 0, GamepadButton::Back);

            self.left_stick = input::get_gamepad_stick_position(ctx, 0, GamepadStick::LeftStick);
            self.right_stick = input::get_gamepad_stick_position(ctx, 0, GamepadStick::RightStick);

            if input::get_gamepad_buttons_pressed(ctx, 0).count() > 0 {
                input::start_gamepad_vibration(ctx, 0, 1.0, 0);
            }

            self.axis_info.set_content(format!(
                "Gamepad: {}\nLeft Stick: ({}, {}) | Right Stick: ({}, {}) | Left Trigger: {} | Right Trigger: {}",
                input::get_gamepad_name(ctx, 0).unwrap(),
                input::get_gamepad_axis_position(ctx, 0, GamepadAxis::LeftStickX),
                input::get_gamepad_axis_position(ctx, 0, GamepadAxis::LeftStickY),
                input::get_gamepad_axis_position(ctx, 0, GamepadAxis::RightStickX),
                input::get_gamepad_axis_position(ctx, 0, GamepadAxis::RightStickY),
                input::get_gamepad_axis_position(ctx, 0, GamepadAxis::LeftTrigger),
                input::get_gamepad_axis_position(ctx, 0, GamepadAxis::RightTrigger)
            ));
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        if self.connected {
            self.draw_button(ctx, 1080.0, 400.0, Sprite::A, self.a);
            self.draw_button(ctx, 1180.0, 300.0, Sprite::B, self.b);
            self.draw_button(ctx, 980.0, 300.0, Sprite::X, self.x);
            self.draw_button(ctx, 1080.0, 200.0, Sprite::Y, self.y);

            self.draw_button(ctx, 100.0, 200.0, Sprite::Up, self.up);
            self.draw_button(ctx, 100.0, 400.0, Sprite::Down, self.down);
            self.draw_button(ctx, 0.0, 300.0, Sprite::Left, self.left);
            self.draw_button(ctx, 200.0, 300.0, Sprite::Right, self.right);

            self.draw_button(ctx, 100.0, 100.0, Sprite::LeftBumper, self.lb);
            self.draw_button(ctx, 100.0, 0.0, Sprite::LeftTrigger, self.lt);
            self.draw_button(ctx, 1080.0, 100.0, Sprite::RightBumper, self.rb);
            self.draw_button(ctx, 1080.0, 0.0, Sprite::RightTrigger, self.rt);

            self.draw_button(ctx, 680.0, 500.0, Sprite::Start, self.start);
            self.draw_button(ctx, 500.0, 500.0, Sprite::Back, self.back);

            self.draw_stick(ctx, 300.0, 500.0, Sprite::LeftStick, self.left_stick);
            self.draw_stick(ctx, 880.0, 500.0, Sprite::RightStick, self.right_stick);

            self.axis_info.draw(ctx, Vec2::new(16.0, 720.0 - 48.0));
        } else {
            self.draw_button(ctx, 16.0, 16.0, Sprite::Disconnected, false);
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Gamepad Input", 1280, 720)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
