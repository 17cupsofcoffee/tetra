use glm::Mat4;
use graphics::Texture;
use opengl::{BufferUsage, GLBuffer};
use util;
use App;

pub struct SpriteBatch {
    // GL handles
    vertex_buffer: GLBuffer,
    index_buffer: GLBuffer,

    texture: Texture,

    drawing: bool,
    vertices: Vec<f32>,
    sprite_count: usize,

    projection: Mat4,
}

impl SpriteBatch {
    pub fn new(app: &mut App, texture: Texture) -> SpriteBatch {
        SpriteBatch::with_capacity(app, 1024, texture)
    }

    pub fn with_capacity(app: &mut App, capacity: usize, texture: Texture) -> SpriteBatch {
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

        let vertex_buffer = app
            .gl
            .new_vertex_buffer(capacity, 7 * 4, BufferUsage::DynamicDraw);
        app.gl
            .set_vertex_buffer_attribute(&vertex_buffer, 0, 4, 7, 0);
        app.gl
            .set_vertex_buffer_attribute(&vertex_buffer, 1, 3, 7, 4);

        let index_buffer = app
            .gl
            .new_index_buffer(capacity, 6, BufferUsage::StaticDraw);
        app.gl.set_index_buffer_data(&index_buffer, &indices, 0);

        SpriteBatch {
            vertex_buffer,
            index_buffer,
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

        app.gl
            .set_uniform(&self.texture.shader.handle, "projection", &self.projection);

        app.gl
            .set_vertex_buffer_data(&self.vertex_buffer, &self.vertices, 0);

        app.gl.draw(
            &self.vertex_buffer,
            &self.index_buffer,
            &self.texture.shader.handle,
            &self.texture.handle,
            self.sprite_count,
        );

        self.vertices.clear();
        self.sprite_count = 0;
    }
}
