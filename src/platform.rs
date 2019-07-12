mod opengl;
mod sdl;

pub(crate) use opengl::{
    FramebufferHandle, GraphicsDevice, IndexBufferHandle, ProgramHandle, TextureHandle,
    VertexBufferHandle,
};

pub(crate) use sdl::Platform;
