//! Functions and types relating to text rendering.

use std::cell::RefCell;
use std::fs;
use std::path::Path;

use glyph_brush::rusttype::{Rect, Scale};
use glyph_brush::{BrushAction, BrushError, FontId, GlyphVertex, Section};

use crate::error::Result;
use crate::graphics::opengl::GLDevice;
use crate::graphics::{
    self, ActiveShader, ActiveTexture, DrawParams, Drawable, Rectangle, Texture, TextureFormat,
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
#[derive(Copy, Clone)]
pub struct Font {
    id: FontId,
}

impl Font {
    /// Loads a font from the given file.
    pub fn new<P: AsRef<Path>>(ctx: &mut Context, path: P) -> Result<Font> {
        let font_bytes = fs::read(path)?;
        let id = ctx.graphics.font_cache.add_font_bytes(font_bytes);

        Ok(Font { id })
    }
}

impl Default for Font {
    fn default() -> Font {
        Font {
            id: FontId::default(),
        }
    }
}

/// The cached text bounds.
enum TextBounds {
    /// Bounds are not known and should be recalculated.
    Unknown,

    /// The text has no bounds because it's an empty string.
    NoBounds,

    /// The bounds are the given rectangle.
    Known(Rectangle),
}

/// A piece of text that can be rendered.
pub struct Text {
    content: String,
    font: Font,
    size: Scale,
    bounds: RefCell<TextBounds>,
    quads: RefCell<Vec<FontQuad>>,
}

impl Text {
    /// Creates a new `Text`, with the given content, font and scale.
    pub fn new<S: Into<String>>(content: S, font: Font, size: f32) -> Text {
        let content = content.into();

        Text {
            content,
            font,
            size: Scale::uniform(size),
            bounds: RefCell::new(TextBounds::Unknown),
            quads: RefCell::new(Vec::new()),
        }
    }

    /// Sets the content of the text.
    pub fn set_content<S: Into<String>>(&mut self, content: S) {
        self.content = content.into();
        self.invalidate();
    }

    /// Get the outer bounds of the text when rendered to the screen.
    ///
    /// If the text is not rendered yet, this method will re-render it and calculate the bounds.
    /// The bounds are automatically cached, so calling this multiple times will only render once.
    ///
    /// Note that this method will not take into account the positioning applied to the text via `DrawParams`.
    pub fn get_bounds(&self, ctx: &mut Context) -> Option<Rectangle> {
        match *self.bounds.borrow() {
            TextBounds::Unknown => {}
            TextBounds::NoBounds => return None,
            TextBounds::Known(r) => return Some(r),
        }

        self.attempt_recalculate_quads(ctx);

        match *self.bounds.borrow() {
            TextBounds::Unknown => unreachable!(),
            TextBounds::NoBounds => None,
            TextBounds::Known(r) => Some(r),
        }
    }

    /// Sets the font of the text.
    pub fn set_font(&mut self, font: Font) {
        self.font = font;
        self.invalidate();
    }

    /// Sets the size of the text.
    pub fn set_size(&mut self, size: f32) {
        self.size = Scale::uniform(size);
        self.invalidate();
    }

    /// Invalidates this Text component, forcing it to redraw itself on the next draw pass.
    fn invalidate(&mut self) {
        self.quads = RefCell::new(Vec::new());
        self.bounds = RefCell::new(TextBounds::Unknown);
    }

    /// Attempt to recalculate the font quads.
    fn attempt_recalculate_quads(&self, ctx: &mut Context) {
        let section = Section {
            text: &self.content,
            scale: self.size,
            font_id: self.font.id,
            ..Section::default()
        };

        if let BrushAction::Draw(new_quads) = draw_text(ctx, section) {
            *self.bounds.borrow_mut() = if new_quads.is_empty() {
                TextBounds::NoBounds
            } else {
                let mut max_x = std::f32::MIN;
                let mut max_y = std::f32::MIN;
                let mut min_x = std::f32::MAX;
                let mut min_y = std::f32::MAX;

                for quad in &new_quads {
                    max_x = max_x.max(quad.x1).max(quad.x2);
                    max_y = max_y.max(quad.y1).max(quad.y2);
                    min_x = min_x.min(quad.x1).min(quad.x2);
                    min_y = min_y.min(quad.y1).min(quad.y2);
                }

                TextBounds::Known(graphics::Rectangle::new(
                    min_x,
                    min_y,
                    max_x - min_x,
                    max_y - min_y,
                ))
            };

            *self.quads.borrow_mut() = new_quads;
        }
    }
}

impl Drawable for Text {
    fn draw<T: Into<DrawParams>>(&self, ctx: &mut Context, params: T) {
        let params = params.into();
        let transform = params.build_matrix();

        // TODO: This should probably only be called when self.quads is empty and self.text is not empty?
        self.attempt_recalculate_quads(ctx);

        graphics::set_texture_ex(ctx, ActiveTexture::FontCache);
        graphics::set_shader_ex(ctx, ActiveShader::Text);

        for quad in self.quads.borrow().iter() {
            graphics::push_quad(
                ctx,
                quad.x1,
                quad.y1,
                quad.x2,
                quad.y2,
                quad.u1,
                quad.v1,
                quad.u2,
                quad.v2,
                &transform,
                params.color,
            );
        }
    }
}

fn draw_text(ctx: &mut Context, section: Section) -> BrushAction<FontQuad> {
    ctx.graphics.font_cache.queue(section);

    let screen_dimensions = (
        graphics::get_width(ctx) as u32,
        graphics::get_height(ctx) as u32,
    );

    // to avoid some borrow checker/closure weirdness
    let texture_ref = &mut ctx.graphics.font_cache_texture;
    let device_ref = &mut ctx.gl;

    loop {
        let attempted_action = ctx.graphics.font_cache.process_queued(
            screen_dimensions,
            |rect, data| update_texture(device_ref, texture_ref, rect, data),
            |v| glyph_to_quad(&v),
        );

        if let Err(BrushError::TextureTooSmall { suggested, .. }) = attempted_action {
            let (width, height) = suggested;

            *texture_ref = Texture::from_handle(device_ref.new_texture(
                width as i32,
                height as i32,
                TextureFormat::Red,
            ));

            ctx.graphics.font_cache.resize_texture(width, height);
            continue;
        }

        break attempted_action.unwrap();
    }
}

fn update_texture(gl: &mut GLDevice, texture: &Texture, rect: Rect<u32>, data: &[u8]) {
    gl.set_unpack_alignment(1);
    gl.set_texture_data(
        &texture.handle,
        data,
        rect.min.x as i32,
        rect.min.y as i32,
        rect.width() as i32,
        rect.height() as i32,
        TextureFormat::Red,
    );
    gl.set_unpack_alignment(4);
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
