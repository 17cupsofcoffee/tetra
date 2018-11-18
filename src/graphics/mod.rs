pub mod color;
pub mod opengl;
pub mod shader;
pub mod texture;

use glm::{Mat4, Vec2};

pub use self::color::Color;
pub use self::shader::Shader;
pub use self::texture::Texture;
use graphics::opengl::{BufferUsage, GLDevice, GLIndexBuffer, GLProgram, GLVertexBuffer};
use util;
use Context;

const SPRITE_CAPACITY: usize = 1024;
const VERTEX_STRIDE: usize = 7;
const INDEX_STRIDE: usize = 6;
const INDEX_ARRAY: [u32; INDEX_STRIDE] = [0, 1, 2, 2, 3, 0];
const DEFAULT_VERTEX_SHADER: &str = include_str!("../resources/shader.vert");
const DEFAULT_FRAGMENT_SHADER: &str = include_str!("../resources/shader.frag");

pub struct RenderState {
    vertex_buffer: GLVertexBuffer,
    index_buffer: GLIndexBuffer,
    texture: Option<Texture>,
    shader: Option<Shader>,
    default_shader: GLProgram,
    projection_matrix: Mat4,
    vertices: Vec<f32>,
    sprite_count: usize,
    capacity: usize,
}

impl RenderState {
    pub fn new(device: &mut GLDevice, width: f32, height: f32) -> RenderState {
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

        let vertex_buffer = device.new_vertex_buffer(
            SPRITE_CAPACITY * 4 * VERTEX_STRIDE,
            VERTEX_STRIDE,
            BufferUsage::DynamicDraw,
        );

        device.set_vertex_buffer_attribute(&vertex_buffer, 0, 4, 0);
        device.set_vertex_buffer_attribute(&vertex_buffer, 1, 3, 4);

        let index_buffer =
            device.new_index_buffer(SPRITE_CAPACITY * INDEX_STRIDE, BufferUsage::StaticDraw);

        device.set_index_buffer_data(&index_buffer, &indices, 0);

        let default_shader = device.compile_program(DEFAULT_VERTEX_SHADER, DEFAULT_FRAGMENT_SHADER);

        RenderState {
            vertex_buffer,
            index_buffer,
            texture: None,
            shader: None,
            default_shader,
            projection_matrix: util::ortho(0.0, width, height, 0.0, -1.0, 1.0),
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
            color: color::WHITE,
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

pub trait Drawable {
    fn draw<T: Into<DrawParams>>(&self, ctx: &mut Context, params: T);
}

pub fn clear(ctx: &mut Context, color: Color) {
    ctx.gl.clear(color.r, color.g, color.b, color.a);
}

pub(crate) fn push_vertex(ctx: &mut Context, x: f32, y: f32, u: f32, v: f32, color: Color) {
    ctx.render_state.vertices.push(x);
    ctx.render_state.vertices.push(y);
    ctx.render_state.vertices.push(u);
    ctx.render_state.vertices.push(v);
    ctx.render_state.vertices.push(color.r);
    ctx.render_state.vertices.push(color.g);
    ctx.render_state.vertices.push(color.b);
}

pub fn draw<D: Drawable, P: Into<DrawParams>>(ctx: &mut Context, drawable: &D, params: P) {
    drawable.draw(ctx, params);
}

pub fn set_texture(ctx: &mut Context, texture: &Texture) {
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
}

pub fn flush(ctx: &mut Context) {
    if ctx.render_state.sprite_count > 0 && ctx.render_state.texture.is_some() {
        let shader_handle = ctx
            .render_state
            .shader
            .as_ref()
            .map(|s| &*s.handle)
            .unwrap_or(&ctx.render_state.default_shader);

        ctx.gl.set_uniform(
            shader_handle,
            "projection",
            &ctx.render_state.projection_matrix,
        );

        ctx.gl.set_vertex_buffer_data(
            &ctx.render_state.vertex_buffer,
            &ctx.render_state.vertices,
            0,
        );

        let texture = ctx.render_state.texture.as_ref().unwrap();

        ctx.gl.draw(
            &ctx.render_state.vertex_buffer,
            &ctx.render_state.index_buffer,
            shader_handle,
            &texture.handle,
            ctx.render_state.sprite_count * INDEX_STRIDE,
        );

        ctx.render_state.vertices.clear();
        ctx.render_state.sprite_count = 0;
    }
}
