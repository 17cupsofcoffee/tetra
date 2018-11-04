use gl::{self, types::*};
use glm::Mat4;
use sdl2::{
    video::{GLContext, GLProfile, Window},
    VideoSubsystem,
};
use std::ffi::{CStr, CString};
use std::mem;
use std::ptr;

pub struct GLDevice {
    _ctx: GLContext,

    current_vertex_buffer: GLuint,
    current_index_buffer: GLuint,
    current_program: GLuint,
    current_texture: GLuint,
    current_vertex_array: GLuint,
}

impl GLDevice {
    pub fn new(video: &VideoSubsystem, window: &Window, vsync: bool) -> GLDevice {
        let gl_attr = video.gl_attr();

        // Force Core 3.2 profile - this is reasonably compatible.
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);

        let _ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video.gl_get_proc_address(name) as *const _);

        // Assert we actually got the profile we asked for!
        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 2));

        video.gl_set_swap_interval(if vsync { 1 } else { 0 });

        let mut current_vertex_array = 0;

        unsafe {
            gl::Enable(gl::CULL_FACE);
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

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

            GLDevice {
                _ctx,

                current_vertex_buffer: 0,
                current_index_buffer: 0,
                current_program: 0,
                current_texture: 0,
                current_vertex_array,
            }
        }
    }

    pub fn clear(&mut self, r: f32, g: f32, b: f32, a: f32) {
        unsafe {
            gl::ClearColor(r, g, b, a);
            gl::Clear(gl::COLOR_BUFFER_BIT);
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

    pub fn compile_program(&mut self, vertex_shader: &str, fragment_shader: &str) -> GLProgram {
        unsafe {
            let vertex_buffer = CString::new(vertex_shader).unwrap();
            let fragment_buffer = CString::new(fragment_shader).unwrap();

            let program_id = gl::CreateProgram();

            let vertex_id = gl::CreateShader(gl::VERTEX_SHADER);
            gl::ShaderSource(vertex_id, 1, &vertex_buffer.as_ptr(), ptr::null());
            gl::CompileShader(vertex_id);
            gl::AttachShader(program_id, vertex_id);

            let fragment_id = gl::CreateShader(gl::FRAGMENT_SHADER);
            gl::ShaderSource(fragment_id, 1, &fragment_buffer.as_ptr(), ptr::null());
            gl::CompileShader(fragment_id);
            gl::AttachShader(program_id, fragment_id);

            gl::LinkProgram(program_id);

            gl::DeleteShader(vertex_id);
            gl::DeleteShader(fragment_id);

            GLProgram { id: program_id }
        }
    }

    pub fn set_uniform<T>(&mut self, program: &GLProgram, name: &str, value: T)
    where
        T: SetUniform,
    {
        unsafe {
            self.bind_program(program);

            let c_name = CString::new(name).unwrap();
            let location = gl::GetUniformLocation(program.id, c_name.as_ptr());
            value.set_uniform(location);
        }
    }

    pub fn new_texture(&mut self, width: i32, height: i32) -> GLTexture {
        // TODO: I don't think we need mipmaps?
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);

            let texture = GLTexture { id };

            self.bind_texture(&texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as GLint);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_BASE_LEVEL, 0);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAX_LEVEL, 0);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as GLint, // love 2 deal with legacy apis
                width,
                height,
                0,
                gl::RGBA,
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
    ) {
        unsafe {
            self.bind_texture(texture);

            gl::TexSubImage2D(
                gl::TEXTURE_2D,
                0,
                x,
                y,
                width,
                height,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                data.as_ptr() as *const GLvoid,
            )
        }
    }

    pub fn draw(
        &mut self,
        vertex_buffer: &GLVertexBuffer,
        index_buffer: &GLIndexBuffer,
        program: &GLProgram,
        texture: &GLTexture,
        count: usize,
    ) {
        unsafe {
            self.bind_program(program);
            self.bind_vertex_buffer(vertex_buffer);
            self.bind_index_buffer(index_buffer);
            self.bind_texture(texture);

            gl::DrawElements(
                gl::TRIANGLES,
                count as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    fn bind_vertex_buffer(&mut self, buffer: &GLVertexBuffer) {
        unsafe {
            if self.current_vertex_buffer != buffer.id {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffer.id);
                self.current_vertex_buffer = buffer.id;
            }
        }
    }

    fn bind_index_buffer(&mut self, buffer: &GLIndexBuffer) {
        unsafe {
            if self.current_index_buffer != buffer.id {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer.id);
                self.current_index_buffer = buffer.id;
            }
        }
    }

    fn bind_program(&mut self, program: &GLProgram) {
        unsafe {
            if self.current_program != program.id {
                gl::UseProgram(program.id);
                self.current_program = program.id;
            }
        }
    }

    fn bind_texture(&mut self, texture: &GLTexture) {
        unsafe {
            if self.current_texture != texture.id {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                self.current_texture = texture.id;
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

#[derive(PartialEq)]
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

pub trait SetUniform {
    unsafe fn set_uniform(&self, location: GLint);
}

impl SetUniform for i32 {
    unsafe fn set_uniform(&self, location: GLint) {
        gl::Uniform1i(location, *self);
    }
}

impl SetUniform for Mat4 {
    unsafe fn set_uniform(&self, location: GLint) {
        gl::UniformMatrix4fv(location, 1, gl::FALSE, self.as_slice().as_ptr());
    }
}

impl<'a, T> SetUniform for &'a T
where
    T: SetUniform,
{
    unsafe fn set_uniform(&self, location: GLint) {
        (**self).set_uniform(location);
    }
}

#[derive(PartialEq)]
pub struct GLTexture {
    id: GLuint,
}

impl Drop for GLTexture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
