use tetra::graphics::mesh::{GeometryBuilder, Mesh, ShapeStyle};
use tetra::graphics::{self, Color};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    simple: Mesh,
    complex: Mesh,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        // For simple one-off shapes, `Mesh` has simple constructors.
        let simple = Mesh::circle(ctx, ShapeStyle::Stroke(16.0), Vec2::zero(), 16.0)?;

        // If you want to create a `Mesh` with multiple shapes, there is a `GeometryBuilder`
        // type that lets you do this. You can also use it to create buffers, or generate
        // raw vertex data that you can process further yourself.
        let complex = GeometryBuilder::new()
            // Background
            .set_color(Color::rgb(1.0, 1.0, 0.0))
            .circle(ShapeStyle::Fill, Vec2::zero(), 64.0)?
            // Face
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

        self.simple.draw(ctx, Vec2::new(64.0, 64.0));
        self.complex.draw(ctx, Vec2::new(256.0, 64.0));

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Shape Drawing", 1280, 720)
        .build()?
        .run(GameState::new)
}
