use std::path::Path;
use std::rc::Rc;

use image;

use error::{Result, TetraError};
use graphics::opengl::GLTexture;
use Context;

#[derive(Clone, PartialEq)]
pub struct Texture {
    pub(crate) handle: Rc<GLTexture>,
    pub width: i32,
    pub height: i32,
}

impl Texture {
    pub fn new<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Result<Texture> {
        let image = image::open(path).map_err(TetraError::Image)?.to_rgba();
        let (width, height) = image.dimensions();

        let texture = ctx.gl.new_texture(width as i32, height as i32);
        ctx.gl
            .set_texture_data(&texture, &image, 0, 0, width as i32, height as i32);

        Ok(Texture {
            handle: Rc::new(texture),
            width: width as i32,
            height: height as i32,
        })
    }
}
