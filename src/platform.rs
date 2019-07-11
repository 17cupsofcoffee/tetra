mod sdl;

pub use sdl::SdlPlatform as ActivePlatform;

use glow::Context as GlowContext;

use crate::error::Result;
use crate::graphics::{
    BufferUsage, Canvas, FilterMode, FrontFace, IndexBuffer, Shader, Texture, UniformValue,
    VertexBuffer,
};
use crate::{Context, ContextBuilder};

pub(crate) trait Platform: Sized {
    type GlContext: GlowContext;

    fn new(builder: &ContextBuilder<'_>) -> Result<(Self, Self::GlContext, i32, i32)>;

    fn handle_events(ctx: &mut Context) -> Result;

    fn show_window(&mut self);
    fn hide_window(&mut self);

    fn get_window_title(&self) -> &str;
    fn set_window_title<S>(&mut self, title: S)
    where
        S: AsRef<str>;

    fn get_window_size(&self) -> (i32, i32);
    fn set_window_size(&mut self, width: i32, height: i32);

    fn toggle_fullscreen(&mut self) -> Result;
    fn enable_fullscreen(&mut self) -> Result;
    fn disable_fullscreen(&mut self) -> Result;
    fn is_fullscreen(&self) -> bool;

    fn set_mouse_visible(&mut self, mouse_visible: bool);
    fn is_mouse_visible(&self) -> bool;

    fn swap_buffers(&self);

    fn get_gamepad_name(&self, platform_id: i32) -> String;
    fn is_gamepad_vibration_supported(&self, platform_id: i32) -> bool;
    fn set_gamepad_vibration(&mut self, platform_id: i32, strength: f32);
    fn start_gamepad_vibration(&mut self, platform_id: i32, strength: f32, duration: u32);
    fn stop_gamepad_vibration(&mut self, platform_id: i32);
}

pub(crate) trait GraphicsDevice {
    fn clear(&mut self, r: f32, g: f32, b: f32, a: f32);

    fn front_face(&mut self, front_face: FrontFace);
    fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32);

    fn draw_elements(&mut self, index_buffer: &IndexBuffer, count: i32);

    fn create_vertex_buffer(
        &mut self,
        count: usize,
        stride: usize,
        usage: BufferUsage,
    ) -> Result<VertexBuffer>;
    fn bind_vertex_buffer(&mut self, buffer: Option<&VertexBuffer>);
    fn set_vertex_buffer_attribute(
        &mut self,
        buffer: &VertexBuffer,
        index: u32,
        size: i32,
        offset: usize,
    );
    fn set_vertex_buffer_data(&mut self, buffer: &VertexBuffer, data: &[f32], offset: usize);

    fn create_index_buffer(&mut self, count: usize, usage: BufferUsage) -> Result<IndexBuffer>;
    fn bind_index_buffer(&mut self, buffer: Option<&IndexBuffer>);
    fn set_index_buffer_data(&mut self, buffer: &IndexBuffer, data: &[u32], offset: usize);

    fn create_texture(&mut self, width: i32, height: i32, data: &[u8]) -> Result<Texture>;
    fn create_texture_empty(&mut self, width: i32, height: i32) -> Result<Texture>;
    fn bind_texture(&mut self, texture: Option<&Texture>);
    fn set_texture_data(
        &mut self,
        texture: &Texture,
        data: &[u8],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    );
    fn set_texture_filter_mode(&mut self, texture: &Texture, filter_mode: FilterMode);

    fn create_shader(&mut self, vertex_shader: &str, fragment_shader: &str) -> Result<Shader>;
    fn bind_shader(&mut self, shader: Option<&Shader>);
    fn set_uniform<T>(&mut self, shader: &Shader, name: &str, value: T)
    where
        T: UniformValue;

    fn create_canvas(&mut self, width: i32, height: i32, rebind_previous: bool) -> Result<Canvas>;
    fn bind_canvas(&mut self, canvas: Option<&Canvas>);
    fn attach_texture_to_canvas(
        &mut self,
        canvas: &Canvas,
        texture: &Texture,
        rebind_previous: bool,
    );
}
