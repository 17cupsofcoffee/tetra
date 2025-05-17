use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color, NineSlice, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{window, Context, ContextBuilder, State};

const LABEL: &str = "\
Use the scroll wheel or W/S to read more!

This is a very long string. Too long, in fact, to fit into the box we're rendering! \
We want to be able to exclude any text that overflows the window.

An easy way to do this is to use graphics::set_scissor, which effectively tells the \
GPU to completely ignore any pixels outside of the specified rectangle when drawing. \
By setting the scissor to the size of this window, we can easily add a scroll \
function, without having to write any complex shaders or use expensive canvas \
switches.";

const PANEL_WIDTH: f32 = 320.0;
const PANEL_HEIGHT: f32 = 128.0;

struct GameState {
    panel_texture: Texture,
    panel_config: NineSlice,
    text: Text,
    text_pos: Vec2<f32>,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            panel_texture: Texture::new(ctx, "./examples/resources/panel.png")?,
            panel_config: NineSlice::with_border(Rectangle::new(0.0, 0.0, 32.0, 32.0), 4.0),
            text: Text::wrapped(
                LABEL,
                Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 14.0)?,
                PANEL_WIDTH - 16.0,
            ),
            text_pos: Vec2::new(8.0, 8.0),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        self.text_pos.y += input::get_mouse_wheel_movement(ctx).y * 8.0;

        if input::is_key_down(ctx, Key::W) {
            self.text_pos.y += 2.0;
        }

        if input::is_key_down(ctx, Key::S) {
            self.text_pos.y -= 2.0;
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        let (window_width, window_height) = window::get_size(ctx);

        let panel_pos = Vec2::new(
            (window_width as f32 / 2.0) - (PANEL_WIDTH / 2.0),
            (window_height as f32 / 2.0) - (PANEL_HEIGHT / 2.0),
        )
        .round();

        self.panel_texture.draw_nine_slice(
            ctx,
            &self.panel_config,
            PANEL_WIDTH,
            PANEL_HEIGHT,
            panel_pos,
        );

        graphics::set_scissor(
            ctx,
            Rectangle::new(
                panel_pos.x as i32 + 4,
                panel_pos.y as i32 + 4,
                PANEL_WIDTH as i32 - 8,
                PANEL_HEIGHT as i32 - 8,
            ),
        );

        self.text.draw(ctx, panel_pos + self.text_pos);

        graphics::reset_scissor(ctx);

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Scissor Rectangles", 640, 480)
        .resizable(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
