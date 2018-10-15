use glm::Mat4;
use graphics::opengl::{BufferUsage, GLBuffer};
use graphics::{Shader, Texture};
use util;
use App;

const VERTEX_STRIDE: usize = 7;
const INDEX_STRIDE: usize = 6;
const INDEX_ARRAY: [u32; INDEX_STRIDE] = [0, 1, 2, 2, 3, 0];

pub struct SpriteBatch {
    // GL handles
    vertex_buffer: GLBuffer,
    index_buffer: GLBuffer,

    texture: Texture,
    shader: Shader,

    vertices: Vec<f32>,
    sprite_count: usize,
    capacity: usize,

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

        let indices: Vec<u32> = INDEX_ARRAY
            .iter()
            .cycle()
            .take(capacity * INDEX_STRIDE)
            .enumerate()
            .map(|(i, vertex)| vertex + i as u32 / INDEX_STRIDE as u32 * 4)
            .collect();

        let vertex_buffer =
            app.gl
                .new_vertex_buffer(capacity, VERTEX_STRIDE * 4, BufferUsage::DynamicDraw);

        app.gl
            .set_vertex_buffer_attribute(&vertex_buffer, 0, 4, VERTEX_STRIDE, 0);
        app.gl
            .set_vertex_buffer_attribute(&vertex_buffer, 1, 3, VERTEX_STRIDE, 4);

        let index_buffer = app
            .gl
            .new_index_buffer(capacity, INDEX_STRIDE, BufferUsage::StaticDraw);

        app.gl.set_index_buffer_data(&index_buffer, &indices, 0);

        let (width, height) = app.window.drawable_size();

        SpriteBatch {
            vertex_buffer,
            index_buffer,
            texture,
            shader: Shader::default(app),
            vertices: Vec::with_capacity(capacity * VERTEX_STRIDE),
            sprite_count: 0,
            capacity,
            projection: util::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0),
        }
    }

    pub fn push(&mut self, x: f32, y: f32, width: f32, height: f32) {
        assert!(self.sprite_count < self.capacity, "Spritebatch is full");

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

    pub fn draw(&mut self, app: &mut App) {
        if self.sprite_count > 0 {
            app.gl
                .set_uniform(&self.shader.handle, "projection", &self.projection);

            app.gl
                .set_vertex_buffer_data(&self.vertex_buffer, &self.vertices, 0);

            app.gl.draw(
                &self.vertex_buffer,
                &self.index_buffer,
                &self.shader.handle,
                &self.texture.handle,
                self.sprite_count,
            );

            self.clear();
        }
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.sprite_count = 0;
    }
}
