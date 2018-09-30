use glm::Mat4;
use image;
use opengl::{Buffer, BufferUsage, Program, ProgramBuilder, ShaderType, Texture};
use util;
use App;

pub struct SpriteBatch {
    // GL handles
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    program: Program,
    texture: Texture,

    drawing: bool,
    vertices: Vec<f32>,
    sprite_count: usize,

    projection: Mat4,
}

impl SpriteBatch {
    pub fn new(app: &mut App) -> SpriteBatch {
        SpriteBatch::with_capacity(app, 1024)
    }

    pub fn with_capacity(app: &mut App, capacity: usize) -> SpriteBatch {
        assert!(
            capacity <= 8191,
            "Can't have more than 8191 sprites to a single buffer"
        );

        let indices_template: [u32; 6] = [0, 1, 2, 2, 3, 0];
        let indices: Vec<u32> = indices_template
            .iter()
            .cycle()
            .take(capacity * 6)
            .enumerate()
            .map(|(i, vertex)| vertex + i as u32 / 6 * 4)
            .collect();

        let program = app.graphics.compile_program(
            &ProgramBuilder::new()
                .with_shader(ShaderType::Vertex, "./resources/shader.vert")
                .with_shader(ShaderType::Fragment, "./resources/shader.frag"),
        );
        app.graphics.set_uniform(&program, "sampler1", 0);

        let vertex_buffer =
            app.graphics
                .new_vertex_buffer(capacity, 7 * 4, BufferUsage::DynamicDraw);
        app.graphics
            .set_vertex_buffer_attribute(&vertex_buffer, 0, 4, 7, 0);
        app.graphics
            .set_vertex_buffer_attribute(&vertex_buffer, 1, 3, 7, 4);

        let index_buffer = app
            .graphics
            .new_index_buffer(capacity, 6, BufferUsage::StaticDraw);
        app.graphics
            .set_index_buffer_data(&index_buffer, &indices, 0);

        let image = image::open("./resources/test.png").unwrap().to_rgba();
        let (width, height) = image.dimensions();

        let texture = app.graphics.new_texture(width as i32, height as i32);
        app.graphics
            .set_texture_data(&texture, &image, 0, 0, width as i32, height as i32);

        SpriteBatch {
            vertex_buffer,
            index_buffer,
            program,
            texture,
            drawing: false,
            vertices: Vec::with_capacity(capacity * 5),
            sprite_count: 0,
            projection: util::ortho(0.0, 1280.0, 720.0, 0.0, -1.0, 1.0),
        }
    }

    pub fn begin(&mut self) {
        assert!(!self.drawing, "Spritebatch is already drawing");

        self.drawing = true;
    }

    pub fn draw(&mut self, x: f32, y: f32, width: f32, height: f32) {
        assert!(self.drawing, "Spritebatch is not currently drawing");

        self.vertices.extend_from_slice(&[
            // top left
            x,
            y,
            0.0,
            0.0,
            1.0,
            1.0,
            1.0,
            // bottom left
            x,
            y + height,
            0.0,
            1.0,
            1.0,
            1.0,
            1.0,
            // bottom right
            x + width,
            y + height,
            1.0,
            1.0,
            1.0,
            1.0,
            1.0,
            // top right
            x + width,
            y,
            1.0,
            0.0,
            1.0,
            1.0,
            1.0,
        ]);

        self.sprite_count += 1;
    }

    pub fn end(&mut self, app: &mut App) {
        assert!(self.drawing, "Spritebatch is not currently drawing");

        if self.sprite_count > 0 {
            self.flush(app);
        }

        self.drawing = false;
    }

    pub fn flush(&mut self, app: &mut App) {
        assert!(self.drawing, "Spritebatch is not currently drawing");

        app.graphics
            .set_uniform(&self.program, "projection", &self.projection);

        app.graphics
            .set_vertex_buffer_data(&self.vertex_buffer, &self.vertices, 0);

        app.graphics.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.program,
            &self.texture,
            self.sprite_count,
        );

        self.vertices.clear();
        self.sprite_count = 0;
    }
}
