pub mod color;
pub mod opengl;
pub mod shader;
pub mod texture;

pub use self::color::Color;
pub use self::shader::Shader;
pub use self::texture::Texture;

use self::opengl::{BufferUsage, GLBuffer, GLDevice};
use glm::Vec2;
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
            device.new_vertex_buffer(SPRITE_CAPACITY * 4, VERTEX_STRIDE, BufferUsage::DynamicDraw);

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
            vertices: Vec::with_capacity(SPRITE_CAPACITY * 4 * VERTEX_STRIDE),
            sprite_count: 0,
            capacity: SPRITE_CAPACITY,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Rectangle<T = f32> {
    pub x: T,
    pub y: T,
    pub width: T,
    pub height: T,
}

impl<T> Rectangle<T> {
    pub fn new(x: T, y: T, width: T, height: T) -> Rectangle<T> {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

pub struct DrawParams {
    pub position: Vec2,
    pub scale: Vec2,
    pub color: Color,
    pub clip: Option<Rectangle>,
}

impl Default for DrawParams {
    fn default() -> DrawParams {
        DrawParams {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            color: Color::rgb(1.0, 1.0, 1.0),
            clip: None,
        }
    }
}

impl From<Vec2> for DrawParams {
    fn from(position: Vec2) -> DrawParams {
        DrawParams {
            position,
            ..DrawParams::default()
        }
    }
}

pub fn clear(ctx: &mut Context, color: Color) {
    ctx.gl.clear(color.r, color.g, color.b, color.a);
}

pub fn draw<T: Into<DrawParams>>(ctx: &mut Context, texture: &Texture, params: T) {
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

    let params = params.into();

    let texture_width = texture.width as f32;
    let texture_height = texture.height as f32;
    let clip = params
        .clip
        .unwrap_or_else(|| Rectangle::new(0.0, 0.0, texture_width, texture_height));

    ctx.render_state.vertices.extend_from_slice(&[
        // top left
        params.position.x,
        params.position.y,
        clip.x / texture_width,
        clip.y / texture_height,
        params.color.r,
        params.color.g,
        params.color.b,
        // bottom left
        params.position.x,
        params.position.y + (clip.width * params.scale.x),
        clip.x / texture_width,
        (clip.y + clip.height) / texture_height,
        params.color.r,
        params.color.g,
        params.color.b,
        // bottom right
        params.position.x + (clip.width * params.scale.x),
        params.position.y + (clip.height * params.scale.y),
        (clip.x + clip.width) / texture_width,
        (clip.y + clip.height) / texture_height,
        params.color.r,
        params.color.g,
        params.color.b,
        // top right
        params.position.x + (clip.width * params.scale.x),
        params.position.y,
        (clip.x + clip.width) / texture_width,
        clip.y / texture_height,
        params.color.r,
        params.color.g,
        params.color.b,
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
