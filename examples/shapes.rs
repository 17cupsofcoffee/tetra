use tetra::graphics::{self, Color, GeometryBuilder, Mesh, ShapeStyle};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    simple: Mesh,
    complex: Mesh,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let simple = Mesh::circle(ctx, ShapeStyle::Stroke(16.0), Vec2::zero(), 16.0)?;

        let complex = GeometryBuilder::new()
            .set_color(Color::rgb(1.0, 1.0, 0.0))
            .circle(ShapeStyle::Fill, Vec2::zero(), 64.0)?
            .set_color(Color::BLACK)
            .circle(ShapeStyle::Fill, Vec2::new(-16.0, -16.0), 8.0)?
            .circle(ShapeStyle::Fill, Vec2::new(16.0, -16.0), 8.0)?
            .polyline(8.0, &[Vec2::new(-16.0, 24.0), Vec2::new(16.0, 24.0)])?
            .build_mesh(ctx)?;

        Ok(GameState { simple, complex })
    }
}

impl State for GameState {
    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        graphics::clear(ctx, Color::rgb(0.392, 0.584, 0.929));

        graphics::draw(ctx, &self.simple, Vec2::new(64.0, 64.0));
        graphics::draw(ctx, &self.complex, Vec2::new(256.0, 64.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Shape Drawing", 1280, 720)
        .build()?
        .run(GameState::new)
}
