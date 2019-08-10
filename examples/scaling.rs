use tetra::graphics::ui::NineSlice;
use tetra::graphics::{self, Color, Font, Rectangle, ScreenScaling, Text, Texture, Vec2};
use tetra::input::{self, Key};
use tetra::{Context, ContextBuilder, State};

const LABEL: &str = "Press Space to cycle between scaling modes";
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
const PANEL_WIDTH: f32 = SCREEN_WIDTH - 48.0;
const PANEL_HEIGHT: f32 = 48.0;
const PANEL_X: f32 = (SCREEN_WIDTH / 2.0) - (PANEL_WIDTH / 2.0);
const PANEL_Y: f32 = (SCREEN_HEIGHT / 2.0) - (PANEL_HEIGHT / 2.0);

struct GameState {
    panel: NineSlice,
    text: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let texture = Texture::new(ctx, "./examples/resources/panel.png")?;

        Ok(GameState {
            panel: NineSlice::new(
                texture,
                PANEL_WIDTH,
                PANEL_HEIGHT,
                Rectangle::new(4.0, 4.0, 24.0, 24.0),
            ),
            text: Text::new(
                format!("{}\n{:?}", LABEL, graphics::get_scaling(ctx)),
                Font::default(),
                16.0,
            ),
        })
    }

    fn set_scaling(&mut self, ctx: &mut Context, mode: ScreenScaling) {
        graphics::set_scaling(ctx, mode);
        self.text.set_content(format!("{}\n{:?}", LABEL, mode));
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_pressed(ctx, Key::Space) {
            match graphics::get_scaling(ctx) {
                ScreenScaling::None => self.set_scaling(ctx, ScreenScaling::Stretch),
                ScreenScaling::Stretch => self.set_scaling(ctx, ScreenScaling::ShowAll),
                ScreenScaling::ShowAll => self.set_scaling(ctx, ScreenScaling::ShowAllPixelPerfect),
                ScreenScaling::ShowAllPixelPerfect => self.set_scaling(ctx, ScreenScaling::Crop),
                ScreenScaling::Crop => self.set_scaling(ctx, ScreenScaling::CropPixelPerfect),
                ScreenScaling::CropPixelPerfect => self.set_scaling(ctx, ScreenScaling::Resize),
                ScreenScaling::Resize => self.set_scaling(ctx, ScreenScaling::None),
            }
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context, _dt: f64) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));
        graphics::draw(ctx, &self.panel, Vec2::new(PANEL_X, PANEL_Y));
        graphics::draw(ctx, &self.text, Vec2::new(PANEL_X + 8.0, PANEL_Y + 8.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Screen Scaling", 640, 480)
        .resizable(true)
        .maximized(true)
        .quit_on_escape(true)
        .build()?
        .run_with(GameState::new)
}
