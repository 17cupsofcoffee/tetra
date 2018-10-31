pub mod color;
pub mod opengl;
pub mod shader;
pub mod spritebatch;
pub mod texture;

pub use self::color::Color;
pub use self::shader::Shader;
pub use self::spritebatch::SpriteBatch;
pub use self::texture::Texture;

use App;

pub fn clear(app: &mut App, color: Color) {
    app.gl.clear(color.r, color.g, color.b, color.a);
}
