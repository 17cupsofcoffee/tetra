use std::rc::Rc;

use crate::glm::{self, Mat4};
use crate::graphics::opengl::{GLDevice, GLFramebuffer};
use crate::graphics::{DrawParams, Drawable, Texture};
use crate::Context;

/// A 2D texture that can be used for off-screen rendering.
///
/// This is sometimes referred to as a 'render texture' or 'render target' in other
/// frameworks.
///
/// Canvases can be useful if you want to do some rendering upfront and then cache the result
/// (e.g. a static background), or if you want to apply transformations/shaders to multiple
/// things simultaneously.
#[derive(Debug, Clone, PartialEq)]
pub struct Canvas {
    pub(crate) texture: Texture,
    pub(crate) framebuffer: Rc<GLFramebuffer>,
    pub(crate) projection: Mat4,
}

impl Canvas {
    /// Creates a new canvas.
    pub fn new(ctx: &mut Context, width: i32, height: i32) -> Canvas {
        Canvas::with_device(&mut ctx.gl, width, height)
    }

    pub(crate) fn with_device(device: &mut GLDevice, width: i32, height: i32) -> Canvas {
        let texture = Texture::with_device_empty(device, width, height);
        let framebuffer = device.new_framebuffer();
        device.attach_texture_to_framebuffer(&framebuffer, &texture.handle, true);

        Canvas {
            texture,
            framebuffer: Rc::new(framebuffer),
            projection: glm::ortho(0.0, width as f32, 0.0, height as f32, -1.0, 1.0),
        }
    }

    /// Returns the width of the canvas.
    pub fn width(&self) -> i32 {
        self.texture.width()
    }

    /// Returns the height of the canvas.
    pub fn height(&self) -> i32 {
        self.texture.height()
    }

    /// Returns the canvas' underlying texture.
    pub fn texture(&self) -> &Texture {
        &self.texture
    }
}

impl Drawable for Canvas {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        self.texture.draw(ctx, params)
    }
}
