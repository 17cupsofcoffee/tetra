//! Functions and types relating to text rendering.

// Without this, we'd have conditionally compile a ton more stuff to
// avoid warnings when fonts are disabled:
#![cfg_attr(not(feature = "font_ttf"), allow(unused))]

mod cache;
mod packer;
#[cfg(feature = "font_ttf")]
mod vector;

use std::cell::{RefCell, RefMut};
use std::fmt::{self, Debug, Formatter};
use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::graphics::text::cache::{FontCache, TextGeometry};
use crate::graphics::{self, DrawParams, Drawable, Rectangle};
use crate::Context;

#[cfg(feature = "font_ttf")]
pub use crate::graphics::text::vector::VectorFontBuilder;

/// A font with an associated size, cached on the GPU.
///
/// # Performance
///
/// Creating a `Font` is a relatively expensive operation. If you can, store them in your `State`
/// struct rather than recreating them each frame.
///
/// Cloning a `Font` is a very cheap operation, as the underlying data is shared between the
/// original instance and the clone via [reference-counting](https://doc.rust-lang.org/std/rc/struct.Rc.html).
///
/// # Examples
///
/// The [`text`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/text.rs)
/// example demonstrates how to load a font and then draw some text.
#[derive(Clone)]
pub struct Font {
    data: Rc<RefCell<FontCache>>,
}

impl Font {
    /// Creates a `Font` from a vector font file, with the given size.
    ///
    /// TrueType and OpenType fonts are supported.
    ///
    /// If you want to load multiple sizes of the same font, you can use a
    /// [`VectorFontBuilder`](struct.VectorFontBuilder.html) to avoid loading/parsing
    /// the file multiple times.
    ///
    /// # Errors
    ///
    /// * `TetraError::FailedToLoadAsset` will be returned if the file could not be loaded.
    /// * `TetraError::InvalidFont` will be returned if the font data was invalid.
    /// * `TetraError::PlatformError` will be returned if the GPU cache for the font
    ///   could not be created.
    #[cfg(feature = "font_ttf")]
    pub fn vector<P>(ctx: &mut Context, path: P, size: f32) -> Result<Font>
    where
        P: AsRef<Path>,
    {
        VectorFontBuilder::new(path)?.with_size(ctx, size)
    }
}

impl Debug for Font {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("Font").finish()
    }
}

/// A piece of text that can be rendered.
///
/// # Performance
///
/// The layout of the text is cached after the first time it is calculated, making subsequent
/// rendering of the text much faster.
///
/// Cloning a `Text` is a fairly expensive operation, as it creates an entirely new copy of the
/// object with its own cache.
///
/// # Examples
///
/// The [`text`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/text.rs)
/// example demonstrates how to load a font and then draw some text.
#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    font: Font,
    geometry: RefCell<Option<TextGeometry>>,
}

impl Text {
    /// Creates a new `Text`, with the given content and font.
    pub fn new<C>(content: C, font: Font) -> Text
    where
        C: Into<String>,
    {
        Text {
            content: content.into(),
            font,
            geometry: RefCell::new(None),
        }
    }

    /// Returns a reference to the content of the text.
    pub fn content(&self) -> &str {
        &self.content
    }

    /// Sets the content of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn set_content<C>(&mut self, content: C)
    where
        C: Into<String>,
    {
        self.geometry.replace(None);
        self.content = content.into();
    }

    /// Gets the font of the text.
    pub fn font(&self) -> &Font {
        &self.font
    }

    /// Sets the font of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn set_font(&mut self, font: Font) {
        self.geometry.replace(None);
        self.font = font;
    }

    /// Appends the given character to the end of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn push(&mut self, ch: char) {
        self.geometry.replace(None);
        self.content.push(ch);
    }

    /// Appends the given string slice to the end of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn push_str(&mut self, string: &str) {
        self.geometry.replace(None);
        self.content.push_str(string);
    }

    /// Removes the last character from the text and returns it.
    ///
    /// Returns `None` if the text is empty.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn pop(&mut self) -> Option<char> {
        self.geometry.replace(None);
        self.content.pop()
    }

    /// Get the outer bounds of the text when rendered to the screen.
    ///
    /// If the text's layout needs calculating, this method will do so.
    ///
    /// Note that this method will not take into account the positioning applied to the text via `DrawParams`.
    pub fn get_bounds(&self, ctx: &mut Context) -> Option<Rectangle> {
        let geometry = self.get_latest_geometry(ctx);

        geometry.bounds
    }

    fn get_latest_geometry(&self, ctx: &mut Context) -> RefMut<'_, TextGeometry> {
        let mut data = self.font.data.borrow_mut();
        let mut geometry = self.geometry.borrow_mut();

        let needs_render = match &*geometry {
            None => true,
            Some(g) => g.resize_count != data.resize_count(),
        };

        if needs_render {
            let new_geometry = data.render(&mut ctx.device, &self.content);
            geometry.replace(new_geometry);
        }

        RefMut::map(geometry, |g| {
            g.as_mut()
                .expect("Geometry should have already been generated")
        })
    }
}

impl Drawable for Text {
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        let params = params.into();

        let geometry = self.get_latest_geometry(ctx);

        let data = self.font.data.borrow();
        graphics::set_texture(ctx, data.texture());

        for quad in &geometry.quads {
            graphics::push_quad(
                ctx,
                quad.position.x,
                quad.position.y,
                quad.position.right(),
                quad.position.bottom(),
                quad.uv.x,
                quad.uv.y,
                quad.uv.right(),
                quad.uv.bottom(),
                &params,
            );
        }
    }
}
