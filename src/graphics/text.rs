//! Functions and types relating to text rendering.

use std::cell::RefCell;
use std::fs;
use std::path::Path;

use glyph_brush::rusttype::{Rect, Scale};
use glyph_brush::{BrushAction, BrushError, FontId, GlyphCruncher, GlyphVertex, Section};

use crate::error::Result;
use crate::graphics::opengl::GLDevice;
use crate::graphics::{
    self, ActiveTexture, DrawParams, Drawable, Rectangle, Texture, TextureFormat,
};
use crate::Context;

struct FontQuad {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
    u1: f32,
    v1: f32,
    u2: f32,
    v2: f32,
}

/// A font that can be used to render text. TrueType fonts (.ttf) and a subset of OpenType fonts (.otf)
/// are supported.
///
/// The actual data for fonts is cached in the `Context`, so there should be no overhead for copying
/// this type - as such, it implements `Copy` and `Clone`.
///
/// Deja Vu Sans Mono is provided as a default font, and can be used by calling `Font::default()`.
/// If you use it, you must distribute the license with your game - it can be found in `src/resources`.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Font {
    id: FontId,
}

impl Font {
    /// Loads a font from the given file.
    /// 
    /// # Errors
    /// 
    /// If the file could not be read, a `TetraError::Io` will be returned.
    pub fn new<P>(ctx: &mut Context, path: P) -> Result<Font>
    where
        P: AsRef<Path>,
    {
        let font_bytes = fs::read(path)?;
        let id = ctx.graphics.font_cache.add_font_bytes(font_bytes);

        Ok(Font { id })
    }

    /// Loads a font from a slice of binary data.
    /// 
    /// This is useful in combination with `include_bytes`, as it allows you to
    /// include your fonts directly in the binary.
    /// 
    /// Note that this function currently requires the slice to have the `'static`
    /// lifetime due to the way that the font cache is implemented - this may change
    /// in the future.
    pub fn from_data(ctx: &mut Context, data: &'static [u8]) -> Font {
        let id = ctx.graphics.font_cache.add_font_bytes(data);

        Font { id }
    }
}

impl Default for Font {
    fn default() -> Font {
        Font {
            id: FontId::default(),
        }
    }
}

/// A piece of text that can be rendered.
pub struct Text {
    content: String,
    font: Font,
    size: Scale,
    quads: RefCell<Vec<FontQuad>>,
}

impl Text {
    /// Creates a new `Text`, with the given content, font and scale.
    pub fn new<S>(content: S, font: Font, size: f32) -> Text
    where
        S: Into<String>,
    {
        let content = content.into();

        Text {
            content,
            font,
            size: Scale::uniform(size),
            quads: RefCell::new(Vec::new()),
        }
    }

    /// Sets the content of the text.
    pub fn set_content<S>(&mut self, content: S)
    where
        S: Into<String>,
    {
        self.content = content.into();
    }

    /// Get the outer bounds of the text when rendered to the screen.
    ///
    /// If the text is not rendered yet, this method will re-render it and calculate the bounds.
    /// The bounds are automatically cached, so calling this multiple times will only render once.
    ///
    /// Note that this method will not take into account the positioning applied to the text via `DrawParams`.
    pub fn get_bounds(&self, ctx: &mut Context) -> Option<Rectangle> {
        ctx.graphics
            .font_cache
            .pixel_bounds(self.build_section())
            .map(|r| {
                let x = r.min.x as f32;
                let y = r.min.y as f32;
                let width = r.width() as f32;
                let height = r.height() as f32;

                Rectangle::new(x, y, width, height)
            })
    }

    /// Sets the font of the text.
    pub fn set_font(&mut self, font: Font) {
        self.font = font;
    }

    /// Sets the size of the text.
    pub fn set_size(&mut self, size: f32) {
        self.size = Scale::uniform(size);
    }

    fn build_section(&self) -> Section {
        Section {
            text: &self.content,
            scale: self.size,
            font_id: self.font.id,

            ..Section::default()
        }
    }

    fn check_for_update(&self, ctx: &mut Context) {
        ctx.graphics.font_cache.queue(self.build_section());

        let screen_dimensions = (
            graphics::get_internal_width(ctx) as u32,
            graphics::get_internal_height(ctx) as u32,
        );

        // to avoid some borrow checker/closure weirdness
        let texture_ref = &mut ctx.graphics.font_cache_texture;
        let device_ref = &mut ctx.gl;

        let action = loop {
            let attempted_action = ctx.graphics.font_cache.process_queued(
                screen_dimensions,
                |rect, data| update_texture(device_ref, texture_ref, rect, data),
                |v| glyph_to_quad(&v),
            );

            match attempted_action {
                Ok(action) => break action,
                Err(BrushError::TextureTooSmall { suggested, .. }) => {
                    let (width, height) = suggested;

                    *texture_ref = Texture::from_handle(device_ref.new_texture(
                        width as i32,
                        height as i32,
                        TextureFormat::Rgba,
                    ));

                    ctx.graphics.font_cache.resize_texture(width, height);
                }
            }
        };

        if let BrushAction::Draw(new_quads) = action {
            *self.quads.borrow_mut() = new_quads;
        }
    }
}

impl Drawable for Text {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let params = params.into();

        self.check_for_update(ctx);

        graphics::set_texture_ex(ctx, ActiveTexture::FontCache);

        for quad in self.quads.borrow().iter() {
            graphics::push_quad(
                ctx, quad.x1, quad.y1, quad.x2, quad.y2, quad.u1, quad.v1, quad.u2, quad.v2,
                &params,
            );
        }
    }
}

fn update_texture(gl: &mut GLDevice, texture: &Texture, rect: Rect<u32>, data: &[u8]) {
    let mut padded_data = Vec::with_capacity(data.len() * 4);

    for a in data {
        padded_data.push(255);
        padded_data.push(255);
        padded_data.push(255);
        padded_data.push(*a);
    }

    gl.set_texture_data(
        &texture.handle,
        &padded_data,
        rect.min.x as i32,
        rect.min.y as i32,
        rect.width() as i32,
        rect.height() as i32,
        TextureFormat::Rgba,
    );
}

fn glyph_to_quad(v: &GlyphVertex) -> FontQuad {
    FontQuad {
        x1: v.pixel_coords.min.x as f32,
        y1: v.pixel_coords.min.y as f32,
        x2: v.pixel_coords.max.x as f32,
        y2: v.pixel_coords.max.y as f32,
        u1: v.tex_coords.min.x as f32,
        v1: v.tex_coords.min.y as f32,
        u2: v.tex_coords.max.x as f32,
        v2: v.tex_coords.max.y as f32,
    }
}
