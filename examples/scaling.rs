use tetra::graphics::scaling::{ScalingMode, ScreenScaler};
use tetra::graphics::text::{Font, Text};
use tetra::graphics::{self, Color, NineSlice, Rectangle, Texture};
use tetra::input::{self, Key};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, Event, State};

const LABEL: &str = "Press Space to cycle between scaling modes";
const SCREEN_WIDTH: f32 = 640.0;
const SCREEN_HEIGHT: f32 = 480.0;
const PANEL_WIDTH: f32 = SCREEN_WIDTH - 48.0;
const PANEL_HEIGHT: f32 = 48.0;
const PANEL_X: f32 = (SCREEN_WIDTH / 2.0) - (PANEL_WIDTH / 2.0);
const PANEL_Y: f32 = (SCREEN_HEIGHT / 2.0) - (PANEL_HEIGHT / 2.0);

struct GameState {
    scaler: ScreenScaler,
    panel_texture: Texture,
    panel_config: NineSlice,
    text: Text,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        Ok(GameState {
            scaler: ScreenScaler::with_window_size(ctx, 640, 480, ScalingMode::Fixed)?,
            panel_texture: Texture::new(ctx, "./examples/resources/panel.png")?,
            panel_config: NineSlice::with_border(Rectangle::new(0.0, 0.0, 32.0, 32.0), 4.0),
            text: Text::new(
                format!("{}\n{:?}", LABEL, ScalingMode::Fixed),
                Font::vector(ctx, "./examples/resources/DejaVuSansMono.ttf", 14.0)?,
            ),
        })
    }

    fn set_mode(&mut self, mode: ScalingMode) {
        self.scaler.set_mode(mode);
        self.text.set_content(format!("{}\n{:?}", LABEL, mode));
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_pressed(ctx, Key::Space) {
            let next = match self.scaler.mode() {
                ScalingMode::Fixed => ScalingMode::Stretch,
                ScalingMode::Stretch => ScalingMode::ShowAll,
                ScalingMode::ShowAll => ScalingMode::ShowAllPixelPerfect,
                ScalingMode::ShowAllPixelPerfect => ScalingMode::Crop,
                ScalingMode::Crop => ScalingMode::CropPixelPerfect,
                ScalingMode::CropPixelPerfect => ScalingMode::Fixed,
                _ => ScalingMode::Fixed,
            };

            self.set_mode(next);
        }

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::set_canvas(ctx, self.scaler.canvas());
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        self.panel_texture.draw_nine_slice(
            ctx,
            &self.panel_config,
            PANEL_WIDTH,
            PANEL_HEIGHT,
            Vec2::new(PANEL_X, PANEL_Y),
        );
        self.text.draw(ctx, Vec2::new(PANEL_X + 8.0, PANEL_Y + 8.0));

        graphics::reset_canvas(ctx);
        graphics::clear(ctx, Color::BLACK);

        self.scaler.draw(ctx);

        Ok(())
    }

    fn event(&mut self, _: &mut Context, event: Event) -> tetra::Result {
        if let Event::Resized { width, height } = event {
            self.scaler.set_outer_size(width, height);
        }

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Screen Scaling", 640, 480)
        .resizable(true)
        .maximized(true)
        .quit_on_escape(true)
        .build()?
        .run(GameState::new)
}
