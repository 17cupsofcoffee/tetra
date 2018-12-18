use tetra::glm::Vec2;
use tetra::graphics::{self, Color, Font, NineSlice, Rectangle, ScreenScaling, Text, Texture};
use tetra::input::{self, Key};
use tetra::{self, Context, ContextBuilder, State};

const LABEL: &str = "Press Space to cycle between scaling modes\nScreenScaling::";
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
                LABEL.to_owned() + "ShowAllPixelPerfect",
                Font::default(),
                16.0,
            ),
        })
    }
}

impl State for GameState {
    fn update(&mut self, ctx: &mut Context) -> tetra::Result {
        if input::is_key_pressed(ctx, Key::Space) {
            match graphics::get_scaling(ctx) {
                ScreenScaling::None => {
                    graphics::set_scaling(ctx, ScreenScaling::Stretch);
                    self.text.set_content(LABEL.to_owned() + "Stretch");
                }
                ScreenScaling::Stretch => {
                    graphics::set_scaling(ctx, ScreenScaling::ShowAll);
                    self.text.set_content(LABEL.to_owned() + "ShowAll");
                }
                ScreenScaling::ShowAll => {
                    graphics::set_scaling(ctx, ScreenScaling::ShowAllPixelPerfect);
                    self.text
                        .set_content(LABEL.to_owned() + "ShowAllPixelPerfect");
                }
                ScreenScaling::ShowAllPixelPerfect => {
                    graphics::set_scaling(ctx, ScreenScaling::Crop);
                    self.text.set_content(LABEL.to_owned() + "Crop");
                }
                ScreenScaling::Crop => {
                    graphics::set_scaling(ctx, ScreenScaling::CropPixelPerfect);
                    self.text.set_content(LABEL.to_owned() + "CropPixelPerfect");
                }
                ScreenScaling::CropPixelPerfect => {
                    graphics::set_scaling(ctx, ScreenScaling::Resize);
                    self.text.set_content(LABEL.to_owned() + "Resize");
                }
                ScreenScaling::Resize => {
                    graphics::set_scaling(ctx, ScreenScaling::None);
                    self.text.set_content(LABEL.to_owned() + "None");
                }
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
    let ctx = &mut ContextBuilder::new()
        .title("Screen Scaling")
        .size(640, 480)
        .resizable(true)
        .maximized(true)
        .quit_on_escape(true)
        .build()?;

    let state = &mut GameState::new(ctx)?;

    tetra::run(ctx, state)
}
