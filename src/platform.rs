mod opengl;
mod sdl;

pub(crate) use opengl::{
    GLDevice as GraphicsDevice, GLFramebuffer as FramebufferHandle,
    GLIndexBuffer as IndexBufferHandle, GLProgram as ProgramHandle, GLTexture as TextureHandle,
    GLVertexBuffer as VertexBufferHandle,
};

pub(crate) use sdl::SdlPlatform as Platform;
