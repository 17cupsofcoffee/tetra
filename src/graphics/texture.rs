use graphics::opengl::GLTexture;
use image;
use std::path::Path;
use std::rc::Rc;
use Context;

#[derive(Clone)]
pub struct Texture {
    pub(crate) handle: Rc<GLTexture>,
}

impl Texture {
    pub fn new<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Texture {
        let image = image::open(path).unwrap().to_rgba();
        let (width, height) = image.dimensions();

        let texture = ctx.gl.new_texture(width as i32, height as i32);
        ctx.gl
            .set_texture_data(&texture, &image, 0, 0, width as i32, height as i32);

        Texture {
            handle: Rc::new(texture),
        }
    }
}
