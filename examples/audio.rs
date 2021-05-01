use tetra::audio::{self, Sound, SoundInstance};
use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

const INSTRUCTIONS: &str = "\
Press Space to 'fire and forget' a sound.

There are also three SoundInstances that can be controlled
independently:

|           | Play | Pause | Stop | Toggle Repeating |
| Channel 1 | Q    | W     | E    | R                |
| Channel 2 | A    | S     | D    | F                |
| Channel 3 | Z    | X     | C    | V                |
";

struct GameState {
    text: Text,
    sound: Sound,
    channel1: SoundInstance,
    channel2: SoundInstance,
    channel3: SoundInstance,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        audio::set_master_volume(ctx, 0.4);

        let sound = Sound::new("./examples/resources/powerup.ogg")?;
        let channel1 = sound.spawn(ctx)?;
        let channel2 = sound.spawn(ctx)?;
        let channel3 = sound.spawn(ctx)?;

        Ok(GameState {
            text: Text::new(
                INSTRUCTIONS,
                Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 16.0)?,
            ),
            sound,
            channel1,
            channel2,
            channel3,
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        for key in input::get_keys_pressed(ctx) {
            match key {
                Key::Space => {
                    self.sound.play(ctx)?;
                }

                Key::Q => self.channel1.play(),
                Key::W => self.channel1.pause(),
                Key::E => self.channel1.stop(),
                Key::R => self.channel1.toggle_repeating(),

                Key::A => self.channel2.play(),
                Key::S => self.channel2.pause(),
                Key::D => self.channel2.stop(),
                Key::F => self.channel2.toggle_repeating(),

                Key::Z => self.channel3.play(),
                Key::X => self.channel3.pause(),
                Key::C => self.channel3.stop(),
                Key::V => self.channel3.toggle_repeating(),

                _ => {}
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.text.draw(ctx, Vec2::new(16.0, 16.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Audio Playback", 640, 480)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
