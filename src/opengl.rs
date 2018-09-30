use gl::{self, types::*};
use glm::Mat4;
use sdl2::{
    video::{GLContext, GLProfile, Window},
    VideoSubsystem,
};
use std::ffi::{CStr, CString};
use std::fs::File;
use std::io::prelude::*;
use std::mem;
use std::path::Path;
use std::ptr;

pub struct OpenGLDevice {
    _ctx: GLContext,

    current_vertex_buffer: GLuint,
    current_index_buffer: GLuint,
    current_program: GLuint,
    current_texture: GLuint,
    current_vertex_array: GLuint,
}

impl OpenGLDevice {
    pub fn new(video: &VideoSubsystem, window: &Window) -> OpenGLDevice {
        let gl_attr = video.gl_attr();

        // Force Core 3.2 profile - this is reasonably compatible.
        gl_attr.set_context_profile(GLProfile::Core);
        gl_attr.set_context_version(3, 2);

        let _ctx = window.gl_create_context().unwrap();
        gl::load_with(|name| video.gl_get_proc_address(name) as *const _);

        // Assert we actually got the profile we asked for!
        debug_assert_eq!(gl_attr.context_profile(), GLProfile::Core);
        debug_assert_eq!(gl_attr.context_version(), (3, 2));

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

            OpenGLDevice {
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

    pub fn new_vertex_buffer(&mut self, count: usize, stride: usize, usage: BufferUsage) -> Buffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);

            let buffer = Buffer { id, count, stride };

            self.bind_vertex_buffer(&buffer);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (buffer.size() * mem::size_of::<GLfloat>()) as GLsizeiptr,
                ptr::null() as *const GLvoid,
                usage.into(),
            );

            buffer
        }
    }

    pub fn set_vertex_buffer_attribute(
        &mut self,
        buffer: &Buffer,
        index: u32,
        size: i32,
        stride: usize,
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
                (stride * mem::size_of::<GLfloat>()) as GLsizei,
                (offset * mem::size_of::<GLfloat>()) as *const _,
            );

            gl::EnableVertexAttribArray(index);
        }
    }

    pub fn set_vertex_buffer_data(&mut self, buffer: &Buffer, data: &[GLfloat], offset: usize) {
        unsafe {
            assert!(offset + data.len() <= buffer.size());

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

    pub fn new_index_buffer(&mut self, count: usize, stride: usize, usage: BufferUsage) -> Buffer {
        unsafe {
            let mut id = 0;
            gl::GenBuffers(1, &mut id);

            let buffer = Buffer { id, count, stride };

            self.bind_index_buffer(&buffer);

            gl::BufferData(
                gl::ELEMENT_ARRAY_BUFFER,
                (buffer.size() * mem::size_of::<GLuint>()) as GLsizeiptr,
                ptr::null() as *const GLvoid,
                usage.into(),
            );

            buffer
        }
    }

    pub fn set_index_buffer_data(&mut self, buffer: &Buffer, data: &[GLuint], offset: usize) {
        unsafe {
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

    pub fn compile_program(&mut self, program: &ProgramBuilder) -> Program {
        unsafe {
            let id = gl::CreateProgram();
            let mut shader_ids = Vec::with_capacity(program.shaders.len());

            for (shader, shader_type) in &program.shaders {
                let shader_id = gl::CreateShader((*shader_type).into());
                gl::ShaderSource(shader_id, 1, &shader.as_ptr(), ptr::null());
                gl::CompileShader(shader_id);
                gl::AttachShader(id, shader_id);

                shader_ids.push(shader_id);
            }

            gl::LinkProgram(id);

            for shader_id in shader_ids {
                gl::DeleteShader(shader_id);
            }

            Program { id }
        }
    }

    pub fn set_uniform<T>(&mut self, program: &Program, name: &str, value: T)
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

    pub fn new_texture(&mut self, width: i32, height: i32) -> Texture {
        // TODO: I don't think we need mipmaps?
        unsafe {
            let mut id = 0;
            gl::GenTextures(1, &mut id);

            let texture = Texture { id };

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
        texture: &Texture,
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
        vertex_buffer: &Buffer,
        index_buffer: &Buffer,
        program: &Program,
        texture: &Texture,
        count: usize,
    ) {
        unsafe {
            self.bind_program(program);
            self.bind_vertex_buffer(vertex_buffer);
            self.bind_index_buffer(index_buffer);
            self.bind_texture(texture);

            gl::DrawElements(
                gl::TRIANGLES,
                (count * index_buffer.stride) as GLsizei,
                gl::UNSIGNED_INT,
                ptr::null(),
            );
        }
    }

    fn bind_vertex_buffer(&mut self, buffer: &Buffer) {
        unsafe {
            if self.current_vertex_buffer != buffer.id {
                gl::BindBuffer(gl::ARRAY_BUFFER, buffer.id);
                self.current_vertex_buffer = buffer.id;
            }
        }
    }

    fn bind_index_buffer(&mut self, buffer: &Buffer) {
        unsafe {
            if self.current_index_buffer != buffer.id {
                gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, buffer.id);
                self.current_index_buffer = buffer.id;
            }
        }
    }

    fn bind_program(&mut self, program: &Program) {
        unsafe {
            if self.current_program != program.id {
                gl::UseProgram(program.id);
                self.current_program = program.id;
            }
        }
    }

    fn bind_texture(&mut self, texture: &Texture) {
        unsafe {
            if self.current_texture != texture.id {
                gl::ActiveTexture(gl::TEXTURE0);
                gl::BindTexture(gl::TEXTURE_2D, texture.id);
                self.current_texture = texture.id;
            }
        }
    }
}

impl Drop for OpenGLDevice {
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

pub struct Buffer {
    id: GLuint,
    count: usize,
    stride: usize,
}

impl Buffer {
    fn size(&self) -> usize {
        self.count * self.stride
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.id);
        }
    }
}

#[derive(Clone, Copy)]
pub enum ShaderType {
    Vertex,
    Fragment,
}

impl From<ShaderType> for GLenum {
    fn from(shader_type: ShaderType) -> GLenum {
        match shader_type {
            ShaderType::Vertex => gl::VERTEX_SHADER,
            ShaderType::Fragment => gl::FRAGMENT_SHADER,
        }
    }
}

#[derive(Default)]
pub struct ProgramBuilder {
    shaders: Vec<(CString, ShaderType)>,
}

impl ProgramBuilder {
    pub fn new() -> ProgramBuilder {
        ProgramBuilder {
            shaders: Vec::with_capacity(2),
        }
    }

    pub fn with_shader<P>(mut self, shader_type: ShaderType, path: P) -> ProgramBuilder
    where
        P: AsRef<Path>,
    {
        let mut shader_file = File::open(path).unwrap();
        let mut buffer = String::new();
        shader_file.read_to_string(&mut buffer).unwrap();
        let c_buffer = CString::new(buffer).unwrap();
        self.shaders.push((c_buffer, shader_type));
        self
    }
}

pub struct Program {
    id: GLuint,
}

impl Drop for Program {
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

pub struct Texture {
    id: GLuint,
}

impl Drop for Texture {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.id);
        }
    }
}
