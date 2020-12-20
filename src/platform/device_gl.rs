use std::cell::Cell;
use std::mem;
use std::rc::Rc;

use glow::{Context as GlowContext, HasContext, PixelUnpackData};

use crate::error::{Result, TetraError};
use crate::graphics::{BufferUsage, FilterMode, VertexWinding};
use crate::math::{Mat2, Mat3, Mat4, Vec2, Vec3, Vec4};

/// Utility function for calculating offsets/sizes.
fn size<T>(elements: usize) -> i32 {
    (elements * mem::size_of::<T>()) as i32
}

type BufferId = <GlowContext as HasContext>::Buffer;
type ProgramId = <GlowContext as HasContext>::Program;
type TextureId = <GlowContext as HasContext>::Texture;
type FramebufferId = <GlowContext as HasContext>::Framebuffer;
type VertexArrayId = <GlowContext as HasContext>::VertexArray;

pub type UniformLocation = <GlowContext as HasContext>::UniformLocation;

#[derive(Debug)]
struct GraphicsState {
    gl: GlowContext,

    current_vertex_buffer: Cell<Option<BufferId>>,
    current_index_buffer: Cell<Option<BufferId>>,
    current_program: Cell<Option<ProgramId>>,
    current_textures: Vec<Cell<Option<TextureId>>>,
    current_framebuffer: Cell<Option<FramebufferId>>,
    current_vertex_array: Cell<Option<VertexArrayId>>,
}

pub struct GraphicsDevice {
    state: Rc<GraphicsState>,
}

impl GraphicsDevice {
    pub fn new(gl: GlowContext) -> Result<GraphicsDevice> {
        unsafe {
            gl.enable(glow::CULL_FACE);
            gl.enable(glow::BLEND);

            // This default might want to change if we introduce
            // custom blending modes.
            gl.blend_func_separate(
                glow::SRC_ALPHA,
                glow::ONE_MINUS_SRC_ALPHA,
                glow::ONE,
                glow::ONE_MINUS_SRC_ALPHA,
            );

            // This is only needed for Core GL - if we wanted to be uber compatible, we'd
            // turn it off on older versions.
            let current_vertex_array = gl
                .create_vertex_array()
                .map_err(TetraError::PlatformError)?;

            gl.bind_vertex_array(Some(current_vertex_array));

            // TODO: Find a nice way of exposing this via the platform layer
            // println!("Swap Interval: {:?}", video.gl_get_swap_interval());

            let texture_units =
                gl.get_parameter_i32(glow::MAX_COMBINED_TEXTURE_IMAGE_UNITS) as usize;

            let state = GraphicsState {
                gl,

                current_vertex_buffer: Cell::new(None),
                current_index_buffer: Cell::new(None),
                current_program: Cell::new(None),
                current_textures: vec![Cell::new(None); texture_units],
                current_framebuffer: Cell::new(None),
                current_vertex_array: Cell::new(Some(current_vertex_array)),
            };

            Ok(GraphicsDevice {
                state: Rc::new(state),
            })
        }
    }

    pub fn get_renderer(&self) -> String {
        unsafe { self.state.gl.get_parameter_string(glow::RENDERER) }
    }

    pub fn get_version(&self) -> String {
        unsafe { self.state.gl.get_parameter_string(glow::VERSION) }
    }

    pub fn get_vendor(&self) -> String {
        unsafe { self.state.gl.get_parameter_string(glow::VENDOR) }
    }

    pub fn get_shading_language_version(&self) -> String {
        unsafe {
            self.state
                .gl
                .get_parameter_string(glow::SHADING_LANGUAGE_VERSION)
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            self.state.gl.clear_color(r, g, b, a);
            self.state.gl.clear(glow::COLOR_BUFFER_BIT);
        }
    }

    pub fn front_face(&mut self, front_face: VertexWinding) {
        unsafe {
            self.state.gl.front_face(front_face.into());
        }
    }

    pub fn new_vertex_buffer(
        &mut self,
        count: usize,
        stride: usize,
        usage: BufferUsage,
    ) -> Result<RawVertexBuffer> {
        unsafe {
            let id = self
                .state
                .gl
                .create_buffer()
                .map_err(TetraError::PlatformError)?;

            let buffer = RawVertexBuffer {
                state: Rc::clone(&self.state),
                id,
                count,
                stride,
            };

            self.bind_vertex_buffer(Some(&buffer));

            self.state.gl.buffer_data_size(
                glow::ARRAY_BUFFER,
                size::<f32>(buffer.size()),
                usage.into(),
            );

            Ok(buffer)
        }
    }

    pub fn set_vertex_buffer_data(
        &mut self,
        buffer: &RawVertexBuffer,
        data: &[f32],
        offset: usize,
    ) {
        self.bind_vertex_buffer(Some(buffer));

        assert!(
            data.len() + offset <= buffer.size(),
            "tried to write out of bounds buffer data"
        );

        unsafe {
            // TODO: What if we want to discard what's already there?

            self.state.gl.buffer_sub_data_u8_slice(
                glow::ARRAY_BUFFER,
                size::<f32>(offset),
                bytemuck::cast_slice(data),
            );
        }
    }

    pub fn new_index_buffer(&mut self, count: usize, usage: BufferUsage) -> Result<RawIndexBuffer> {
        unsafe {
            let id = self
                .state
                .gl
                .create_buffer()
                .map_err(TetraError::PlatformError)?;

            let buffer = RawIndexBuffer {
                state: Rc::clone(&self.state),
                id,
                count,
            };

            self.bind_index_buffer(Some(&buffer));

            self.state.gl.buffer_data_size(
                glow::ELEMENT_ARRAY_BUFFER,
                size::<u32>(count),
                usage.into(),
            );

            Ok(buffer)
        }
    }

    pub fn set_index_buffer_data(&mut self, buffer: &RawIndexBuffer, data: &[u32], offset: usize) {
        self.bind_index_buffer(Some(buffer));

        assert!(
            data.len() + offset <= buffer.count(),
            "tried to write out of bounds buffer data"
        );

        unsafe {
            // TODO: What if we want to discard what's already there?

            self.state.gl.buffer_sub_data_u8_slice(
                glow::ELEMENT_ARRAY_BUFFER,
                size::<u32>(offset),
                bytemuck::cast_slice(data),
            );
        }
    }

    pub fn new_program(
        &mut self,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<RawProgram> {
        unsafe {
            let program_id = self
                .state
                .gl
                .create_program()
                .map_err(TetraError::PlatformError)?;

            // TODO: IDK if this should be applied to *all* shaders...
            self.state
                .gl
                .bind_attrib_location(program_id, 0, "a_position");
            self.state.gl.bind_attrib_location(program_id, 1, "a_uv");
            self.state.gl.bind_attrib_location(program_id, 2, "a_color");

            let vertex_id = self
                .state
                .gl
                .create_shader(glow::VERTEX_SHADER)
                .map_err(TetraError::PlatformError)?;

            self.state.gl.shader_source(vertex_id, vertex_shader);
            self.state.gl.compile_shader(vertex_id);
            self.state.gl.attach_shader(program_id, vertex_id);

            if !self.state.gl.get_shader_compile_status(vertex_id) {
                return Err(TetraError::InvalidShader(
                    self.state.gl.get_shader_info_log(vertex_id),
                ));
            }

            let fragment_id = self
                .state
                .gl
                .create_shader(glow::FRAGMENT_SHADER)
                .map_err(TetraError::PlatformError)?;

            self.state.gl.shader_source(fragment_id, fragment_shader);
            self.state.gl.compile_shader(fragment_id);
            self.state.gl.attach_shader(program_id, fragment_id);

            if !self.state.gl.get_shader_compile_status(fragment_id) {
                return Err(TetraError::InvalidShader(
                    self.state.gl.get_shader_info_log(fragment_id),
                ));
            }

            self.state.gl.link_program(program_id);

            if !self.state.gl.get_program_link_status(program_id) {
                return Err(TetraError::InvalidShader(
                    self.state.gl.get_program_info_log(program_id),
                ));
            }

            self.state.gl.delete_shader(vertex_id);
            self.state.gl.delete_shader(fragment_id);

            let program = RawProgram {
                state: Rc::clone(&self.state),
                id: program_id,
            };

            let sampler_location = self.get_uniform_location(&program, "u_texture");
            self.set_uniform_i32(&program, sampler_location.as_ref(), 0);

            Ok(program)
        }
    }

    pub fn get_uniform_location(
        &self,
        program: &RawProgram,
        name: &str,
    ) -> Option<UniformLocation> {
        unsafe { self.state.gl.get_uniform_location(program.id, name) }
    }

    pub fn set_uniform_i32(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: i32,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state.gl.uniform_1_i32(location, value);
        }
    }

    pub fn set_uniform_u32(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: u32,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state.gl.uniform_1_u32(location, value);
        }
    }

    pub fn set_uniform_f32(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: f32,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state.gl.uniform_1_f32(location, value);
        }
    }

    pub fn set_uniform_vec2(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: Vec2<f32>,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state
                .gl
                .uniform_2_f32_slice(location, &value.into_array());
        }
    }

    pub fn set_uniform_vec3(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: Vec3<f32>,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state
                .gl
                .uniform_3_f32_slice(location, &value.into_array());
        }
    }

    pub fn set_uniform_vec4(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: Vec4<f32>,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state
                .gl
                .uniform_4_f32_slice(location, &value.into_array());
        }
    }

    pub fn set_uniform_mat2(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: Mat2<f32>,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state.gl.uniform_matrix_2_f32_slice(
                location,
                value.gl_should_transpose(),
                &value.into_col_array(),
            );
        }
    }

    pub fn set_uniform_mat3(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: Mat3<f32>,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state.gl.uniform_matrix_3_f32_slice(
                location,
                value.gl_should_transpose(),
                &value.into_col_array(),
            );
        }
    }

    pub fn set_uniform_mat4(
        &mut self,
        program: &RawProgram,
        location: Option<&UniformLocation>,
        value: Mat4<f32>,
    ) {
        self.bind_program(Some(program));

        unsafe {
            self.state.gl.uniform_matrix_4_f32_slice(
                location,
                value.gl_should_transpose(),
                &value.into_col_array(),
            );
        }
    }

    pub fn new_texture(&mut self, width: i32, height: i32) -> Result<RawTexture> {
        // TODO: I don't think we need mipmaps?
        unsafe {
            let id = self
                .state
                .gl
                .create_texture()
                .map_err(TetraError::PlatformError)?;

            let texture = RawTexture {
                state: Rc::clone(&self.state),

                id,
                width,
                height,
            };

            self.bind_default_texture(Some(&texture));

            self.state.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_S,
                glow::CLAMP_TO_EDGE as i32,
            );

            self.state.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_WRAP_T,
                glow::CLAMP_TO_EDGE as i32,
            );

            self.state
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_BASE_LEVEL, 0);

            self.state
                .gl
                .tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAX_LEVEL, 0);

            self.state.gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32, // love 2 deal with legacy apis
                width,
                height,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );

            Ok(texture)
        }
    }

    pub fn set_texture_data(
        &mut self,
        texture: &RawTexture,
        data: &[u8],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result {
        assert!(
            x >= 0 && y >= 0 && x + width <= texture.width && y + height <= texture.height,
            "tried to write outside of texture bounds"
        );

        let expected = (width * height * 4) as usize;
        let actual = data.len();

        if expected > actual {
            return Err(TetraError::NotEnoughData { expected, actual });
        }

        self.bind_default_texture(Some(texture));

        unsafe {
            self.state.gl.tex_sub_image_2d(
                glow::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelUnpackData::Slice(data),
            )
        }

        Ok(())
    }

    pub fn set_texture_filter_mode(&mut self, texture: &RawTexture, filter_mode: FilterMode) {
        self.bind_default_texture(Some(texture));

        unsafe {
            self.state.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MIN_FILTER,
                filter_mode.into(),
            );

            self.state.gl.tex_parameter_i32(
                glow::TEXTURE_2D,
                glow::TEXTURE_MAG_FILTER,
                filter_mode.into(),
            );
        }
    }

    pub fn new_framebuffer(
        &mut self,
        texture: &RawTexture,
        rebind_previous: bool,
    ) -> Result<RawFramebuffer> {
        unsafe {
            let id = self
                .state
                .gl
                .create_framebuffer()
                .map_err(TetraError::PlatformError)?;

            let framebuffer = RawFramebuffer {
                state: Rc::clone(&self.state),
                id,
            };

            let previous_id = self.state.current_framebuffer.get();

            self.bind_framebuffer(Some(&framebuffer));

            self.state.gl.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture.id),
                0,
            );

            if rebind_previous {
                self.state
                    .gl
                    .bind_framebuffer(glow::FRAMEBUFFER, previous_id);
                self.state.current_framebuffer.set(previous_id);
            }

            Ok(framebuffer)
        }
    }

    pub fn viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            self.state.gl.viewport(x, y, width, height);
        }
    }

    pub fn draw_arrays(
        &mut self,
        vertex_buffer: &RawVertexBuffer,
        texture: &RawTexture,
        program: &RawProgram,
        offset: usize,
        count: usize,
    ) {
        self.bind_vertex_buffer(Some(vertex_buffer));
        self.bind_default_texture(Some(texture));
        self.bind_program(Some(program));

        let max_count = vertex_buffer.size() / vertex_buffer.stride();

        let offset = usize::min(offset, max_count.saturating_sub(1));
        let count = usize::min(count, max_count.saturating_sub(offset));

        unsafe {
            self.state
                .gl
                .draw_arrays(glow::TRIANGLES, offset as i32, count as i32);
        }
    }

    pub fn draw_elements(
        &mut self,
        vertex_buffer: &RawVertexBuffer,
        index_buffer: &RawIndexBuffer,
        texture: &RawTexture,
        program: &RawProgram,
        offset: usize,
        count: usize,
    ) {
        self.bind_vertex_buffer(Some(vertex_buffer));
        self.bind_index_buffer(Some(index_buffer));
        self.bind_default_texture(Some(texture));
        self.bind_program(Some(program));

        let max_count = index_buffer.count();

        let offset = usize::min(offset, max_count.saturating_sub(1));
        let count = usize::min(count, max_count.saturating_sub(offset));

        unsafe {
            self.state.gl.draw_elements(
                glow::TRIANGLES,
                count as i32,
                glow::UNSIGNED_INT,
                size::<u32>(offset),
            );
        }
    }

    fn bind_vertex_buffer(&mut self, buffer: Option<&RawVertexBuffer>) {
        unsafe {
            let id = buffer.map(|x| x.id);

            if self.state.current_vertex_buffer.get() != id {
                self.state.gl.bind_buffer(glow::ARRAY_BUFFER, id);

                // TODO: This only works because we don't let the user set custom
                // attribute bindings - will need a rethink at that point!
                match buffer {
                    Some(b) => {
                        self.state.gl.vertex_attrib_pointer_f32(
                            0,
                            2,
                            glow::FLOAT,
                            false,
                            size::<f32>(b.stride),
                            0,
                        );

                        self.state.gl.vertex_attrib_pointer_f32(
                            1,
                            2,
                            glow::FLOAT,
                            false,
                            size::<f32>(b.stride),
                            size::<f32>(2),
                        );

                        self.state.gl.vertex_attrib_pointer_f32(
                            2,
                            4,
                            glow::FLOAT,
                            false,
                            size::<f32>(b.stride),
                            size::<f32>(4),
                        );

                        self.state.gl.enable_vertex_attrib_array(0);
                        self.state.gl.enable_vertex_attrib_array(1);
                        self.state.gl.enable_vertex_attrib_array(2);
                    }
                    None => {
                        self.state.gl.disable_vertex_attrib_array(0);
                        self.state.gl.disable_vertex_attrib_array(1);
                        self.state.gl.disable_vertex_attrib_array(2);
                    }
                }

                self.state.current_vertex_buffer.set(id);
            }
        }
    }

    fn bind_index_buffer(&mut self, buffer: Option<&RawIndexBuffer>) {
        unsafe {
            let id = buffer.map(|x| x.id);

            if self.state.current_index_buffer.get() != id {
                self.state.gl.bind_buffer(glow::ELEMENT_ARRAY_BUFFER, id);
                self.state.current_index_buffer.set(id);
            }
        }
    }

    fn bind_program(&mut self, program: Option<&RawProgram>) {
        unsafe {
            let id = program.map(|x| x.id);

            if self.state.current_program.get() != id {
                self.state.gl.use_program(id);
                self.state.current_program.set(id);
            }
        }
    }

    pub fn bind_texture(&mut self, texture: Option<&RawTexture>, unit: u32) -> Result {
        unsafe {
            let id = texture.map(|x| x.id);

            let current = &self
                .state
                .current_textures
                .get(unit as usize)
                .ok_or_else(|| TetraError::PlatformError("invalid texture unit".into()))?;

            if current.get() != id {
                self.state.gl.active_texture(glow::TEXTURE0 + unit);
                self.state.gl.bind_texture(glow::TEXTURE_2D, id);
                current.set(id);
            }
        }

        Ok(())
    }

    pub fn bind_default_texture(&mut self, texture: Option<&RawTexture>) {
        self.bind_texture(texture, 0)
            .expect("texture unit 0 should always be available");
    }

    pub fn bind_framebuffer(&mut self, framebuffer: Option<&RawFramebuffer>) {
        unsafe {
            let id = framebuffer.map(|x| x.id);

            if self.state.current_framebuffer.get() != id {
                self.state.gl.bind_framebuffer(glow::FRAMEBUFFER, id);
                self.state.current_framebuffer.set(id);
            }
        }
    }
}

impl Drop for GraphicsDevice {
    fn drop(&mut self) {
        unsafe {
            self.state.gl.bind_vertex_array(None);

            if let Some(va) = self.state.current_vertex_array.get() {
                self.state.gl.delete_vertex_array(va);
            }
        }
    }
}

#[doc(hidden)]
impl From<BufferUsage> for u32 {
    fn from(buffer_usage: BufferUsage) -> u32 {
        match buffer_usage {
            BufferUsage::Static => glow::STATIC_DRAW,
            BufferUsage::Dynamic => glow::DYNAMIC_DRAW,
            BufferUsage::Stream => glow::STREAM_DRAW,
        }
    }
}

#[doc(hidden)]
impl From<VertexWinding> for u32 {
    fn from(front_face: VertexWinding) -> u32 {
        match front_face {
            VertexWinding::Clockwise => glow::CW,
            VertexWinding::CounterClockwise => glow::CCW,
        }
    }
}

#[doc(hidden)]
impl From<FilterMode> for i32 {
    fn from(filter_mode: FilterMode) -> i32 {
        match filter_mode {
            FilterMode::Nearest => glow::NEAREST as i32,
            FilterMode::Linear => glow::LINEAR as i32,
        }
    }
}

macro_rules! handle_impls {
    ($name:ty) => {
        impl PartialEq for $name {
            fn eq(&self, other: &$name) -> bool {
                self.id == other.id
            }
        }
    };
}

#[derive(Debug)]
pub struct RawVertexBuffer {
    state: Rc<GraphicsState>,
    id: BufferId,

    count: usize,
    stride: usize,
}

impl RawVertexBuffer {
    pub fn count(&self) -> usize {
        self.count
    }

    pub fn stride(&self) -> usize {
        self.stride
    }

    pub fn size(&self) -> usize {
        self.count * self.stride
    }
}

impl Drop for RawVertexBuffer {
    fn drop(&mut self) {
        unsafe {
            if self.state.current_vertex_buffer.get() == Some(self.id) {
                self.state.current_vertex_buffer.set(None);
            }

            self.state.gl.delete_buffer(self.id);
        }
    }
}

handle_impls!(RawVertexBuffer);

#[derive(Debug)]
pub struct RawIndexBuffer {
    state: Rc<GraphicsState>,
    id: BufferId,

    count: usize,
}

impl RawIndexBuffer {
    pub fn count(&self) -> usize {
        self.count
    }
}

impl Drop for RawIndexBuffer {
    fn drop(&mut self) {
        unsafe {
            if self.state.current_index_buffer.get() == Some(self.id) {
                self.state.current_index_buffer.set(None);
            }

            self.state.gl.delete_buffer(self.id);
        }
    }
}

handle_impls!(RawIndexBuffer);

#[derive(Debug)]
pub struct RawProgram {
    state: Rc<GraphicsState>,
    id: ProgramId,
}

impl Drop for RawProgram {
    fn drop(&mut self) {
        unsafe {
            if self.state.current_program.get() == Some(self.id) {
                self.state.current_program.set(None);
            }

            self.state.gl.delete_program(self.id);
        }
    }
}

handle_impls!(RawProgram);

#[derive(Debug)]
pub struct RawTexture {
    state: Rc<GraphicsState>,
    id: TextureId,

    width: i32,
    height: i32,
}

impl RawTexture {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}

impl Drop for RawTexture {
    fn drop(&mut self) {
        unsafe {
            for bound in &self.state.current_textures {
                if bound.get() == Some(self.id) {
                    bound.set(None);
                }
            }

            self.state.gl.delete_texture(self.id);
        }
    }
}

handle_impls!(RawTexture);

#[derive(Debug)]
pub struct RawFramebuffer {
    state: Rc<GraphicsState>,
    id: FramebufferId,
}

impl Drop for RawFramebuffer {
    fn drop(&mut self) {
        unsafe {
            if self.state.current_framebuffer.get() == Some(self.id) {
                self.state.current_framebuffer.set(None);
            }

            self.state.gl.delete_framebuffer(self.id);
        }
    }
}

handle_impls!(RawFramebuffer);
