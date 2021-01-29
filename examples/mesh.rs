use tetra::graphics::mesh::{BufferUsage, Mesh, Vertex, VertexBuffer};
use tetra::graphics::{self, Color, DrawParams, Texture};
use tetra::math::Vec2;
use tetra::{Context, ContextBuilder, State};

struct GameState {
    mesh: Mesh,
    timer: f32,
}

impl GameState {
    fn new(ctx: &mut Context) -> tetra::Result<GameState> {
        let (pos_a, uv_a) = (Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
        let (pos_b, uv_b) = (Vec2::new(0.0, 128.0), Vec2::new(0.0, 1.0));
        let (pos_c, uv_c) = (Vec2::new(128.0, 128.0), Vec2::new(1.0, 1.0));
        let (pos_d, uv_d) = (Vec2::new(128.0, 0.0), Vec2::new(1.0, 0.0));

        let vertices = &[
            // Triangle 1
            Vertex::new(pos_a, uv_a, Color::RED),
            Vertex::new(pos_b, uv_b, Color::GREEN),
            Vertex::new(pos_c, uv_c, Color::BLUE),
            // Triangle 2
            Vertex::new(pos_d, uv_d, Color::WHITE),
            Vertex::new(pos_a, uv_a, Color::WHITE),
            Vertex::new(pos_c, uv_c, Color::WHITE),
        ];

        let mut mesh = VertexBuffer::with_usage(ctx, vertices, BufferUsage::Static)?.into_mesh();

        mesh.set_texture(Texture::new(ctx, "./examples/resources/block.png")?);

        Ok(GameState { mesh, timer: 0.0 })
    }
}

impl State for GameState {
    fn update(&mut self, _: &mut Context) -> tetra::Result {
        self.timer += 0.01;

        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> tetra::Result {
        let curve = self.timer.sin() + 2.0;

        graphics::clear(ctx, Color::rgb(0.094, 0.11, 0.16));

        graphics::draw(
            ctx,
            &self.mesh,
            DrawParams::new()
                .position(Vec2::new(1280.0 / 2.0, 720.0 / 2.0))
                .origin(Vec2::new(64.0, 64.0))
                .scale(Vec2::new(curve, curve))
                .rotation(self.timer),
        );

        Ok(())
    }
}

fn main() -> tetra::Result {
    ContextBuilder::new("Custom Mesh", 1280, 720)
        .build()?
        .run(GameState::new)
}
