//! Functions and types relating to text rendering.

mod bmfont;
mod cache;
mod packer;
#[cfg(feature = "font_ttf")]
mod vector;

use std::cell::RefCell;
use std::fmt::{self, Debug, Formatter};
use std::path::Path;
use std::rc::Rc;

use crate::error::Result;
use crate::graphics::text::cache::{FontCache, TextGeometry};
use crate::graphics::{self, DrawParams, Rectangle};
use crate::Context;

#[cfg(feature = "font_ttf")]
pub use crate::graphics::text::vector::VectorFontBuilder;

pub use crate::graphics::text::bmfont::BmFontBuilder;

use super::FilterMode;

/// Different ways that font textures can be generated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FontTextureStyle {
    /// An RGBA texture will be used, with the RGB channels set to 1.0, and
    /// the alpha channels set to the amount of coverage.
    Normal,

    /// An RGBA texture will be used, with all channels set to the amount
    /// of coverage. This will require the [`BlendState`](crate::graphics::BlendState)
    /// to be configured for premultiplied alpha.
    Premultiplied,
}

/// A font with an associated size, cached on the GPU.
///
/// # Performance
///
/// Loading a font is quite an expensive operation, as it involves parsing the font itself and
/// creating a cache on the GPU for the rendered characters. Try to reuse fonts, rather than
/// recreating them every frame.
///
/// You can clone a font cheaply, as it is [reference-counted](https://doc.rust-lang.org/std/rc/struct.Rc.html)
/// internally. However, this does mean that modifying a font (e.g. setting the
/// filter mode) will also affect any clones that exist of it.
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
    /// [`VectorFontBuilder`] to avoid loading/parsing the file multiple times.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`](crate::TetraError::FailedToLoadAsset) will be returned
    /// if the file could not be loaded.
    /// * [`TetraError::InvalidFont`](crate::TetraError::InvalidFont) will be returned if the font
    /// data was invalid.
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the GPU cache for the font
    ///   could not be created.
    #[cfg(feature = "font_ttf")]
    pub fn vector<P>(ctx: &mut Context, path: P, size: f32) -> Result<Font>
    where
        P: AsRef<Path>,
    {
        VectorFontBuilder::new(path)?.with_size(ctx, size)
    }

    /// Creates a `Font` from a slice of binary data.
    ///
    /// TrueType and OpenType fonts are supported.
    ///
    /// This is useful in combination with [`include_bytes`](std::include_bytes), as it
    /// allows you to include your font data directly in the binary.
    ///
    /// If you want to load multiple sizes of the same font, you can use a
    /// [`VectorFontBuilder`] to avoid parsing the data multiple times.
    ///
    /// # Errors
    ///
    /// * [`TetraError::InvalidFont`](crate::TetraError::InvalidFont) will be returned if the font
    /// data was invalid.
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the GPU cache for the font
    /// could not be created.
    #[cfg(feature = "font_ttf")]
    pub fn from_vector_file_data(
        ctx: &mut Context,
        data: &'static [u8],
        size: f32,
    ) -> Result<Font> {
        VectorFontBuilder::from_file_data(data)?.with_size(ctx, size)
    }

    /// Creates a `Font` from an AngelCode BMFont file.
    ///
    /// By default, Tetra will search for the font's images relative to the font itself.
    /// If you need more control over the search path, or want to override the paths
    /// entirely, this can be done via [`BmFontBuilder`].
    ///
    /// Currently, only the text format is supported. Support for the binary file
    /// format may be added in the future.
    ///
    /// # Exporting from BMFont
    ///
    /// For best results, follow these guidelines when exporting from BMFont:
    ///
    /// ## Font Settings
    ///
    /// * For the sizing to match the TTF version of the same font, tick 'match char height'.
    ///
    /// ## Export Options
    ///
    /// * Unless you are using a custom shader, choose the 'white text with alpha' preset.
    /// * Export using the 'text' font descriptor format.
    /// * Make sure the corresponding Tetra feature flag is enabled for your texture's
    ///   file format.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`](crate::TetraError::FailedToLoadAsset) will be returned
    /// if the font or the associated images could not be loaded.
    /// * [`TetraError::InvalidFont`](crate::TetraError::InvalidFont) will be returned if the font
    /// data was invalid.
    /// * [`TetraError::PlatformError`](crate::TetraError::PlatformError) will be returned if the GPU cache for the font
    /// could not be created.
    pub fn bmfont<P>(ctx: &mut Context, path: P) -> Result<Font>
    where
        P: AsRef<Path>,
    {
        BmFontBuilder::new(path)?.build(ctx)
    }

    /// Returns the filter mode of the font.
    pub fn filter_mode(&self) -> FilterMode {
        self.data.borrow().filter_mode()
    }

    /// Sets the filter mode of the font.
    ///
    /// Note that changing the filter mode of a font will affect all [`Text`] objects
    /// that use that font, including existing ones. This is due to the fact that
    /// each font has a shared texture atlas.
    pub fn set_filter_mode(&mut self, ctx: &mut Context, filter_mode: FilterMode) {
        self.data.borrow_mut().set_filter_mode(ctx, filter_mode);
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
/// The layout and geometry of the text is cached after the first time it is
/// calculated, making subsequent renders much faster. If your text stays
/// the same from frame to frame, reusing the `Text` object will be much
/// faster than recreating it.
///
/// # Examples
///
/// The [`text`](https://github.com/17cupsofcoffee/tetra/blob/main/examples/text.rs)
/// example demonstrates how to load a font and then draw some text.
#[derive(Debug, Clone)]
pub struct Text {
    content: String,
    font: Font,
    max_width: Option<f32>,
    geometry: Option<TextGeometry>,
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
            max_width: None,
            geometry: None,
        }
    }

    /// Creates a new wrapped `Text`, with the given content, font
    /// and maximum width.
    ///
    /// If a word is too long to fit, it may extend beyond the max width - use
    /// [`get_bounds`](Text::get_bounds) if you need to find the actual bounds
    /// of the text.
    pub fn wrapped<C>(content: C, font: Font, max_width: f32) -> Text
    where
        C: Into<String>,
    {
        Text {
            content: content.into(),
            font,
            max_width: Some(max_width),
            geometry: None,
        }
    }

    /// Draws the text to the screen (or to a canvas, if one is enabled).
    pub fn draw<P>(&mut self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>,
    {
        self.update_geometry(ctx);

        let params = params.into();

        let data = self.font.data.borrow();
        let texture = data.texture();
        let geometry = self
            .geometry
            .as_ref()
            .expect("geometry should have been generated");

        graphics::set_texture(ctx, texture);
        let (texture_width, texture_height) = texture.size();

        for quad in &geometry.quads {
            graphics::push_quad(
                ctx,
                quad.position.x,
                quad.position.y,
                quad.position.x + quad.region.width,
                quad.position.y + quad.region.height,
                quad.region.x / (texture_width as f32),
                quad.region.y / (texture_height as f32),
                quad.region.right() / (texture_width as f32),
                quad.region.bottom() / (texture_height as f32),
                &params,
            );
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
        self.geometry.take();
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
        self.geometry.take();
        self.font = font;
    }

    /// Gets the maximum width of the text, if one is set.
    ///
    /// If a word is too long to fit, it may extend beyond this width - use
    /// [`get_bounds`](Text::get_bounds) if you need to find the actual bounds
    /// of the text.
    pub fn max_width(&self) -> Option<f32> {
        self.max_width
    }

    /// Sets the maximum width of the text.
    ///
    /// If `Some` is passed, word-wrapping will be enabled. If `None` is passed,
    /// it will be disabled.
    ///
    /// If a word is too long to fit, it may extend beyond this width - use
    /// [`get_bounds`](Text::get_bounds) if you need to find the actual bounds
    /// of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn set_max_width(&mut self, max_width: Option<f32>) {
        self.geometry.take();
        self.max_width = max_width;
    }

    /// Appends the given character to the end of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn push(&mut self, ch: char) {
        self.geometry.take();
        self.content.push(ch);
    }

    /// Appends the given string slice to the end of the text.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn push_str(&mut self, string: &str) {
        self.geometry.take();
        self.content.push_str(string);
    }

    /// Removes the last character from the text and returns it.
    ///
    /// Returns [`None`] if the text is empty.
    ///
    /// Calling this function will cause a re-layout of the text the next time it
    /// is rendered.
    pub fn pop(&mut self) -> Option<char> {
        self.geometry.take();
        self.content.pop()
    }

    /// Get the outer bounds of the text when rendered to the screen.
    ///
    /// If the text's layout needs calculating, this method will do so.
    ///
    /// Note that this method will not take into account the positioning applied to the text via [`DrawParams`].
    pub fn get_bounds(&mut self, ctx: &mut Context) -> Option<Rectangle> {
        self.update_geometry(ctx);

        self.geometry
            .as_ref()
            .expect("geometry should have been generated")
            .bounds
    }

    fn update_geometry(&mut self, ctx: &mut Context) {
        let mut data = self.font.data.borrow_mut();

        let needs_render = match &self.geometry {
            None => true,
            Some(g) => g.resize_count != data.resize_count(),
        };

        if needs_render {
            let new_geometry = data.render(&mut ctx.device, &self.content, self.max_width);
            self.geometry = Some(new_geometry);
        }
    }
}
