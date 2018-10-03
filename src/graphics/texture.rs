use image;
use opengl::GLTexture;
use std::path::Path;
use std::rc::Rc;
use App;

#[derive(Clone)]
pub struct Texture {
    pub(crate) handle: Rc<GLTexture>,
}

impl Texture {
    pub fn new<P: AsRef<Path>>(app: &mut App, path: P) -> Texture {
        let image = image::open(path).unwrap().to_rgba();
        let (width, height) = image.dimensions();

        let texture = app.gl.new_texture(width as i32, height as i32);
        app.gl
            .set_texture_data(&texture, &image, 0, 0, width as i32, height as i32);

        Texture {
            handle: Rc::new(texture),
        }
    }
}
