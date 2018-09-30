use graphics::shader::Shader;
use image;
use opengl::GLTexture;
use std::path::Path;
use App;

pub struct Texture {
    pub(crate) handle: GLTexture,

    pub shader: Shader,
}

impl Texture {
    pub fn new<P: AsRef<Path>>(app: &mut App, path: P, shader: Shader) -> Texture {
        let image = image::open(path).unwrap().to_rgba();
        let (width, height) = image.dimensions();

        let handle = app.gl.new_texture(width as i32, height as i32);
        app.gl
            .set_texture_data(&handle, &image, 0, 0, width as i32, height as i32);

        Texture { handle, shader }
    }
}
