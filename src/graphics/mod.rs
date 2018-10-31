pub mod color;
pub mod opengl;
pub mod shader;
pub mod spritebatch;
pub mod texture;

pub use self::color::Color;
pub use self::shader::Shader;
pub use self::spritebatch::SpriteBatch;
pub use self::texture::Texture;

use Context;

pub fn clear(ctx: &mut Context, color: Color) {
    ctx.gl.clear(color.r, color.g, color.b, color.a);
}
