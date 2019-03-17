use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;

use gl::{self, types::*};
use glm::Mat4;
use sdl2::video::{GLContext, Window};
use sdl2::VideoSubsystem;

use crate::error::{Result, TetraError};

pub struct GLDevice {
    _ctx: GLContext,

    current_vertex_buffer: GLuint,
    current_index_buffer: GLuint,
    current_program: GLuint,
    current_texture: GLuint,
    current_framebuffer: GLuint,
    current_vertex_array: GLuint,
}

impl GLDevice {
    pub fn new(video: &VideoSubsystem, window: &Window, vsync: bool) -> Result<GLDevice> {
        let _ctx = window.gl_create_context().map_err(TetraError::OpenGl)?;
        gl::load_with(|name| video.gl_get_proc_address(name) as *const _);

        video
            .gl_set_swap_interval(if vsync { 1 } else { 0 })
            .map_err(TetraError::Sdl)?;

        let mut current_vertex_array = 0;

        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);

            // This default might want to change if we introduce
            // custom blending modes.
            gl::BlendFuncSeparate(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
                gl::ONE,
                gl::ONE_MINUS_SRC_ALPHA,
            );

            // This is only needed for Core GL - if we wanted to be uber compatible, we'd
            // turn it off on older versions.
            gl::GenVertexArrays(1, &mut current_vertex_array);
            gl::BindVertexArray(current_vertex_array);

            println!(
                "OpenGL Device: {}",
                CStr::from_ptr(gl::GetString(gl::RENDERER) as *const _).to_string_lossy()
            );

            println!(
                "OpenGL Driver: {}",
                CStr::from_ptr(gl::GetString(gl::VERSION) as *const _).to_string_lossy()
            );

            println!(
                "OpenGL Vendor: {}",
                CStr::from_ptr(gl::GetString(gl::VENDOR) as *const _).to_string_lossy()
            );

            println!("Swap Interval: {:?}", video.gl_get_swap_interval());

            Ok(GLDevice {
                _ctx,

                current_vertex_buffer: 0,
                current_index_buffer: 0,
                current_program: 0,
                current_texture: 0,
                current_framebuffer: 0,
                current_vertex_array,
            })
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    pub fn front_face(&mut self, front_face: FrontFace) {
        unsafe {
            gl::FrontFace(front_face.into());
        }
    }

    pub fn new_vertex_buffer(
        &mut self,
        count: usize,
        stride: usize,
        usage: BufferUsage,
    ) -> GLVertexBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);

            let buffer = GLVertexBuffer { id, count, stride };

            self.bind_vertex_buffer(&buffer);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (count * mem::size_of::<GLfloat>()) as GLsizeiptr,
                ptr::null() as *const GLvoid,
                usage.into(),
            );

            buffer
        }
    }

    pub fn set_vertex_buffer_attribute(
        &mut self,
        buffer: &GLVertexBuffer,
        index: u32,
        size: i32,
        offset: usize,
    ) {
        // TODO: This feels a bit unergonomic...

        unsafe {
            self.bind_vertex_buffer(buffer);

            gl::VertexAttribPointer(
                index,
                size,
                gl::FLOAT,
                gl::FALSE,
                (buffer.stride * mem::size_of::<GLfloat>()) as GLsizei,
                (offset * mem::size_of::<GLfloat>()) as *const _,
            );

            gl::EnableVertexAttribArray(index);
        }
    }

    pub fn set_vertex_buffer_data(
        &mut self,
        buffer: &GLVertexBuffer,
        data: &[GLfloat],
        offset: usize,
    ) {
        unsafe {
            assert!(offset + data.len() <= buffer.count);

            self.bind_vertex_buffer(buffer);

            // TODO: What if we want to discard what's already there?

            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                (offset * mem::size_of::<GLfloat>()) as GLsizeiptr,
                (data.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                data.as_ptr() as *const GLvoid,
            );
        }
    }

    pub fn new_index_buffer(&mut self, count: usize, usage: BufferUsage) -> GLIndexBuffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);

            let buffer = GLIndexBuffer { id, count };

            self.bind_index_buffer(&buffer);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (count * mem::size_of::<GLuint>()) as GLsizeiptr,
                ptr::null() as *const GLvoid,
                usage.into(),
            );

            buffer
        }
    }

    pub fn set_index_buffer_data(
        &mut self,
        buffer: &GLIndexBuffer,
        data: &[GLuint],
        offset: usize,
    ) {
        unsafe {
            assert!(offset + data.len() <= buffer.count);

            self.bind_index_buffer(buffer);

            // TODO: What if we want to discard what's already there?

            gl::BufferSubData(
                gl::ELEMENT_ARRAY_BUFFER,
                (offset * mem::size_of::<GLuint>()) as GLsizeiptr,
                (data.len() * mem::size_of::<GLuint>()) as GLsizeiptr,
                data.as_ptr() as *const GLvoid,
            );
        }
    }

    pub fn compile_program(
        &mut self,
        vertex_shader: &str,
        fragment_shader: &str,
    ) -> Result<GLProgram> {
        unsafe {
            let vertex_buffer = CString::new(vertex_shader).unwrap();
            let fragment_buffer = CString::new(fragment_shader).unwrap();

            let program_id = gl::CreateProgram();

            // TODO: IDK if this should be applied to *all* shaders...
            let position_name = CString::new("a_position").unwrap();
            let uv_name = CString::new("a_uv").unwrap();
            let color_name = CString::new("a_color").unwrap();
            let out_color_name = CString::new("o_color").unwrap();

            gl::BindAttribLocation(program_id, 0, position_name.as_ptr());
            gl::BindAttribLocation(program_id, 1, uv_name.as_ptr());
            gl::BindAttribLocation(program_id, 2, color_name.as_ptr());
            gl::BindFragDataLocation(program_id, 0, out_color_name.as_ptr());

            let vertex_id = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_id, 1, &vertex_buffer.as_ptr(), ptr::null());
            gl::CompileShader(vertex_id);
            gl::AttachShader(program_id, vertex_id);

            let mut success = 0;
            gl::GetShaderiv(vertex_id, gl::COMPILE_STATUS, &mut success);

            if success != 1 {
                return Err(TetraError::OpenGl(self.get_shader_info_log(vertex_id)));
            }

            let fragment_id = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment_id, 1, &fragment_buffer.as_ptr(), ptr::null());
            gl::CompileShader(fragment_id);
            gl::AttachShader(program_id, fragment_id);

            let mut success = 0;
            gl::GetShaderiv(fragment_id, gl::COMPILE_STATUS, &mut success);

            if success != 1 {
                return Err(TetraError::OpenGl(self.get_shader_info_log(fragment_id)));
            }

            gl::LinkProgram(program_id);

            let mut success = 0;
            gl::GetProgramiv(program_id, gl::LINK_STATUS, &mut success);

            if success != 1 {
                return Err(TetraError::OpenGl(self.get_program_info_log(program_id)));
            }

            gl::DeleteShader(vertex_id);
            gl::DeleteShader(fragment_id);

            let program = GLProgram { id: program_id };

            self.set_uniform(&program, "u_texture", 0);

            Ok(program)
        }
    }

    fn get_shader_info_log(&mut self, shader_id: GLuint) -> String {
        unsafe {
            let mut max_len = 0;

            gl::GetShaderiv(shader_id, gl::INFO_LOG_LENGTH, &mut max_len);

            let mut result = vec![0u8; max_len as usize];
            let mut result_len = 0 as GLsizei;
            gl::GetShaderInfoLog(
                shader_id,
                max_len as GLsizei,
                &mut result_len,
                result.as_mut_ptr() as *mut GLchar,
            );

            result.truncate(if result_len > 0 {
                result_len as usize
            } else {
                0
            });

            String::from_utf8(result).unwrap()
        }
    }

    fn get_program_info_log(&mut self, program_id: GLuint) -> String {
        unsafe {
            let mut max_len = 0;

            gl::GetProgramiv(program_id, gl::INFO_LOG_LENGTH, &mut max_len);

            let mut result = vec![0u8; max_len as usize];
            let mut result_len = 0 as GLsizei;
            gl::GetProgramInfoLog(
                program_id,
                max_len as GLsizei,
                &mut result_len,
                result.as_mut_ptr() as *mut GLchar,
            );

            result.truncate(if result_len > 0 {
                result_len as usize
            } else {
                0
            });

            String::from_utf8(result).unwrap()
        }
    }

    pub fn set_uniform<T>(&mut self, program: &GLProgram, name: &str, value: T)
    where
        T: UniformValue,
    {
        unsafe {
            self.bind_program(program);

            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(program.id, c_name.as_ptr());
            value.set_uniform(location);
        }
    }

    pub fn new_texture(&mut self, width: i32, height: i32, format: TextureFormat) -> GLTexture {
        // TODO: I don't think we need mipmaps?
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);

            let texture = GLTexture { id, width, height };

            self.bind_texture(&texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);

            let format = format.into();

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                format as GLint, // love 2 deal with legacy apis
                width,
                height,
                0,
                format,
                gl::UNSIGNED_BYTE,
                ptr::null(),
            );

            texture
        }
    }

    pub fn set_texture_data(
        &mut self,
        texture: &GLTexture,
        data: &[u8],
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        format: TextureFormat,
    ) {
        unsafe {
            self.bind_texture(texture);

            let format = format.into();

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                format,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            )
        }
    }

    pub fn new_framebuffer(&mut self) -> GLFramebuffer {
        unsafe {
            let mut id = 0;
            gl::GenFramebuffers(1, &mut id);

            GLFramebuffer { id }
        }
    }

    pub fn attach_texture_to_framebuffer(
        &mut self,
        framebuffer: &GLFramebuffer,
        texture: &GLTexture,
        rebind_previous: bool,
    ) {
        unsafe {
            let previous_id = self.current_framebuffer;

            self.bind_framebuffer(framebuffer);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::COLOR_ATTACHMENT0,
                gl::TEXTURE_2D,
                texture.id,
                0,
            );

            if rebind_previous {
                gl::BindFramebuffer(gl::FRAMEBUFFER, previous_id);
                self.current_framebuffer = previous_id;
            }
        }
    }

    pub fn set_viewport(&mut self, x: i32, y: i32, width: i32, height: i32) {
        unsafe {
            gl::Viewport(x, y, width, height);
        }
    }

    pub fn draw_elements(&mut self, index_buffer: &GLIndexBuffer, count: usize) {
        unsafe {
            self.bind_index_buffer(index_buffer);

            gl::DrawElements(
                gl::TRIANGLES,
                count as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    pub fn bind_vertex_buffer(&mut self, buffer: &GLVertexBuffer) {
        unsafe {
            if self.current_vertex_buffer != buffer.id {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffer.id);
                self.current_vertex_buffer = buffer.id;
            }
        }
    }

    pub fn bind_index_buffer(&mut self, buffer: &GLIndexBuffer) {
        unsafe {
            if self.current_index_buffer != buffer.id {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer.id);
                self.current_index_buffer = buffer.id;
            }
        }
    }

    pub fn bind_program(&mut self, program: &GLProgram) {
        unsafe {
            if self.current_program != program.id {
                gl::UseProgram(program.id);
                self.current_program = program.id;
            }
        }
    }

    pub fn bind_texture(&mut self, texture: &GLTexture) {
        unsafe {
            if self.current_texture != texture.id {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                self.current_texture = texture.id;
            }
        }
    }

    pub fn bind_framebuffer(&mut self, framebuffer: &GLFramebuffer) {
        unsafe {
            if self.current_framebuffer != framebuffer.id {
                gl::BindFramebuffer(gl::FRAMEBUFFER, framebuffer.id);
                self.current_framebuffer = framebuffer.id;
            }
        }
    }

    pub fn bind_default_framebuffer(&mut self) {
        unsafe {
            if self.current_framebuffer != 0 {
                gl::BindFramebuffer(gl::FRAMEBUFFER, 0);
                self.current_framebuffer = 0;
            }
        }
    }
}

impl Drop for GLDevice {
    fn drop(&mut self) {
        unsafe {
            gl::BindVertexArray(0);
            gl::DeleteVertexArrays(1, &self.current_vertex_array);
        }
    }
}

#[derive(Clone, Copy)]
pub enum BufferUsage {
    StaticDraw,
    DynamicDraw,
}

impl From<BufferUsage> for GLenum {
    fn from(buffer_usage: BufferUsage) -> GLenum {
        match buffer_usage {
            BufferUsage::StaticDraw => gl::STATIC_DRAW,
            BufferUsage::DynamicDraw => gl::DYNAMIC_DRAW,
        }
    }
}

#[derive(Clone, Copy)]
pub enum TextureFormat {
    Rgba,
    Rgb,
    Red,
}

impl From<TextureFormat> for GLenum {
    fn from(texture_format: TextureFormat) -> GLenum {
        match texture_format {
            TextureFormat::Rgba => gl::RGBA,
            TextureFormat::Rgb => gl::RGB,
            TextureFormat::Red => gl::RED,
        }
    }
}

#[derive(Clone, Copy)]
pub enum FrontFace {
    Clockwise,
    CounterClockwise,
}

impl From<FrontFace> for GLenum {
    fn from(front_face: FrontFace) -> GLenum {
        match front_face {
            FrontFace::Clockwise => gl::CW,
            FrontFace::CounterClockwise => gl::CCW,
        }
    }
}

pub struct GLVertexBuffer {
    id: GLuint,
    count: usize,
    stride: usize,
}

impl Drop for GLVertexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

#[derive(Debug)]
pub struct GLIndexBuffer {
    id: GLuint,
    count: usize,
}

impl Drop for GLIndexBuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GLProgram {
    id: GLuint,
}

impl Drop for GLProgram {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.id);
        }
    }
}

mod sealed {
    use super::*;
    pub trait UniformValueTypes {}
    impl UniformValueTypes for i32 {}
    impl UniformValueTypes for f32 {}
    impl UniformValueTypes for Mat4 {}
    impl<'a, T> UniformValueTypes for &'a T where T: UniformValueTypes {}
}

/// Represents a type that can be passed as a uniform value to a shader.
///
/// As the implementation of this trait currently interacts directly with the OpenGL layer,
/// it's marked as a [sealed trait](https://rust-lang-nursery.github.io/api-guidelines/future-proofing.html#sealed-traits-protect-against-downstream-implementations-c-sealed),
/// and can't be implemented outside of Tetra. This might change in the future!
pub trait UniformValue: sealed::UniformValueTypes {
    #[doc(hidden)]
    unsafe fn set_uniform(&self, location: GLint);
}

impl UniformValue for i32 {
    #[doc(hidden)]
    unsafe fn set_uniform(&self, location: GLint) {
        gl::Uniform1i(location, *self);
    }
}

impl UniformValue for f32 {
    #[doc(hidden)]
    unsafe fn set_uniform(&self, location: GLint) {
        gl::Uniform1f(location, *self);
    }
}

impl UniformValue for Mat4 {
    #[doc(hidden)]
    unsafe fn set_uniform(&self, location: GLint) {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, self.as_slice().as_ptr());
    }
}

impl<'a, T> UniformValue for &'a T
where
    T: UniformValue,
{
    #[doc(hidden)]
    unsafe fn set_uniform(&self, location: GLint) {
        (**self).set_uniform(location);
    }
}

#[derive(Debug)]
pub struct GLTexture {
    id: GLuint,
    width: i32,
    height: i32,
}

impl GLTexture {
    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}

impl PartialEq for GLTexture {
    fn eq(&self, other: &GLTexture) -> bool {
        self.id == other.id
    }
}

impl Drop for GLTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct GLFramebuffer {
    id: GLuint,
}

impl Drop for GLFramebuffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteFramebuffers(1, &self.id);
        }
    }
}
