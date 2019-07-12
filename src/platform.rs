pub(crate) mod opengl;
mod sdl;

pub(crate) use opengl::GLDevice as GraphicsDevice;
pub(crate) use sdl::SdlPlatform as Platform;
