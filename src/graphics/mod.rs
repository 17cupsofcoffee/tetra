pub mod color;
pub mod opengl;
pub mod shader;
pub mod texture;

pub use self::color::Color;
pub use self::shader::Shader;
pub use self::texture::Texture;

use self::opengl::{BufferUsage, GLBuffer, GLDevice};
use Context;

const SPRITE_CAPACITY: usize = 1024;
const VERTEX_STRIDE: usize = 7;
const INDEX_STRIDE: usize = 6;
const INDEX_ARRAY: [u32; INDEX_STRIDE] = [0, 1, 2, 2, 3, 0];

pub struct RenderState {
    vertex_buffer: GLBuffer,
    index_buffer: GLBuffer,
    texture: Option<Texture>,
    shader: Option<Shader>,
    vertices: Vec<f32>,
    sprite_count: usize,
    capacity: usize,
}

impl RenderState {
    pub fn new(device: &mut GLDevice) -> RenderState {
        assert!(
            SPRITE_CAPACITY <= 8191,
            "Can't have more than 8191 sprites to a single buffer"
        );

        let indices: Vec<u32> = INDEX_ARRAY
            .iter()
            .cycle()
            .take(SPRITE_CAPACITY * INDEX_STRIDE)
            .enumerate()
            .map(|(i, vertex)| vertex + i as u32 / INDEX_STRIDE as u32 * 4)
            .collect();

        let vertex_buffer =
            device.new_vertex_buffer(SPRITE_CAPACITY, VERTEX_STRIDE * 4, BufferUsage::DynamicDraw);

        device.set_vertex_buffer_attribute(&vertex_buffer, 0, 4, VERTEX_STRIDE, 0);
        device.set_vertex_buffer_attribute(&vertex_buffer, 1, 3, VERTEX_STRIDE, 4);

        let index_buffer =
            device.new_index_buffer(SPRITE_CAPACITY, INDEX_STRIDE, BufferUsage::StaticDraw);

        device.set_index_buffer_data(&index_buffer, &indices, 0);

        RenderState {
            vertex_buffer,
            index_buffer,
            texture: None,
            shader: None,
            vertices: Vec::with_capacity(SPRITE_CAPACITY * VERTEX_STRIDE),
            sprite_count: 0,
            capacity: SPRITE_CAPACITY,
        }
    }
}

pub fn clear(ctx: &mut Context, color: Color) {
    ctx.gl.clear(color.r, color.g, color.b, color.a);
}

pub fn draw(ctx: &mut Context, texture: &Texture, x: f32, y: f32) {
    match ctx.render_state.texture {
        Some(ref inner) if inner == texture => {}
        None => {
            ctx.render_state.texture = Some(texture.clone());
        }
        _ => {
            ctx.render_state.texture = Some(texture.clone());
            flush(ctx);
        }
    }

    assert!(
        ctx.render_state.sprite_count < ctx.render_state.capacity,
        "Renderer is full"
    );

    ctx.render_state.vertices.extend_from_slice(&[
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
        y + texture.width as f32,
        0.0,
        1.0,
        1.0,
        1.0,
        1.0,
        // bottom right
        x + texture.width as f32,
        y + texture.height as f32,
        1.0,
        1.0,
        1.0,
        1.0,
        1.0,
        // top right
        x + texture.width as f32,
        y,
        1.0,
        0.0,
        1.0,
        1.0,
        1.0,
    ]);

    ctx.render_state.sprite_count += 1;
}

pub fn flush(ctx: &mut Context) {
    if ctx.render_state.sprite_count > 0 && ctx.render_state.texture.is_some() {
        if ctx.render_state.shader.is_none() {
            // TODO: We only need to compile this once
            ctx.render_state.shader = Some(Shader::default(ctx));
        }

        let texture = ctx.render_state.texture.as_ref().unwrap();
        let shader = ctx.render_state.shader.as_ref().unwrap();

        ctx.gl
            .set_uniform(&shader.handle, "projection", &ctx.projection_matrix);

        ctx.gl.set_vertex_buffer_data(
            &ctx.render_state.vertex_buffer,
            &ctx.render_state.vertices,
            0,
        );

        ctx.gl.draw(
            &ctx.render_state.vertex_buffer,
            &ctx.render_state.index_buffer,
            &shader.handle,
            &texture.handle,
            ctx.render_state.sprite_count,
        );

        ctx.render_state.vertices.clear();
        ctx.render_state.sprite_count = 0;
    }
}
