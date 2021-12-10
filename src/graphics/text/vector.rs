use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use ab_glyph::{Font as AbFont, FontRef, FontVec, PxScale, ScaleFont};

use crate::error::{Result, TetraError};
use crate::fs;
use crate::graphics::text::cache::{FontCache, RasterizedGlyph, Rasterizer};
use crate::graphics::text::{Font, FontTextureStyle};
use crate::graphics::Rectangle;
use crate::math::Vec2;
use crate::Context;

pub(crate) struct VectorRasterizer<F> {
    font: Rc<F>,
    scale: PxScale,
    texture_type: FontTextureStyle,
}

impl<F> VectorRasterizer<F>
where
    F: AbFont,
{
    pub fn new(font: Rc<F>, size: f32, texture_type: FontTextureStyle) -> VectorRasterizer<F> {
        let scale_factor = font
            .units_per_em()
            .map(|units_per_em| font.height_unscaled() / units_per_em)
            .unwrap_or(1.0);

        let px_size = size * scale_factor;
        let scale = PxScale::from(px_size);

        VectorRasterizer {
            font,
            scale,
            texture_type,
        }
    }
}

impl<F> Rasterizer for VectorRasterizer<F>
where
    F: AbFont,
{
    fn rasterize(&self, ch: char, position: Vec2<f32>) -> Option<RasterizedGlyph> {
        let font = self.font.as_scaled(self.scale);

        let mut glyph = font.scaled_glyph(ch);

        glyph.position = ab_glyph::point(position.x, position.y);

        if let Some(outline) = font.outline_glyph(glyph.clone()) {
            let mut data = Vec::new();

            outline.draw(|_, _, v| {
                let coverage = (v * 255.0) as u8;

                data.extend_from_slice(&match self.texture_type {
                    FontTextureStyle::Normal => [255, 255, 255, coverage],
                    FontTextureStyle::Premultiplied => [coverage, coverage, coverage, coverage],
                });
            });

            let bounds = outline.px_bounds();

            Some(RasterizedGlyph {
                data,
                bounds: Rectangle::new(
                    bounds.min.x - glyph.position.x,
                    bounds.min.y - glyph.position.y,
                    bounds.width(),
                    bounds.height(),
                ),
            })
        } else {
            None
        }
    }

    fn advance(&self, glyph: char) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.h_advance(scaled_font.glyph_id(glyph))
    }

    fn line_height(&self) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.height() + scaled_font.line_gap()
    }

    fn ascent(&self) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.ascent()
    }

    fn kerning(&self, previous: char, current: char) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.kern(
            // TODO: This is slow in debug mode
            scaled_font.glyph_id(previous),
            scaled_font.glyph_id(current),
        )
    }
}

/// Abstracts over the two Font types provided by ab_glyph.
///
/// This is preferable to using FontArc because that would incur a double
/// indirection once we type erase the Rasterizer.
#[derive(Debug, Clone)]
enum VectorFontData {
    Owned(Rc<FontVec>),
    Slice(Rc<FontRef<'static>>),
}

/// A builder for vector-based fonts.
///
/// TrueType and OpenType fonts are supported. The font data will only be loaded
/// into memory once, and it will be shared between all [`Font`](struct.Font.html)s that
/// are subsequently created by the builder instance.
///
/// [`Font::vector`] provides a simpler API for loading vector fonts, if you don't need
/// all of the functionality of this struct.
///
/// # Performance
///
/// Creating a `VectorFontBuilder` is a relatively expensive operation. If you need to create
/// extra sizes of the font later on, store the `VectorFontBuilder` rather than building a new one.
///
/// Cloning a `VectorFontBuilder` is a very cheap operation, as the underlying data is shared between the
/// original instance and the clone via [reference-counting](https://doc.rust-lang.org/std/rc/struct.Rc.html).
#[derive(Debug, Clone)]
pub struct VectorFontBuilder {
    data: VectorFontData,
    texture_style: FontTextureStyle,
}

impl VectorFontBuilder {
    /// Loads a vector font from the given file.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if the file could not be loaded.
    /// * [`TetraError::InvalidFont`] will be returned if the font data was invalid.
    pub fn new<P>(path: P) -> Result<VectorFontBuilder>
    where
        P: AsRef<Path>,
    {
        let font_bytes = fs::read(path)?;
        let font = FontVec::try_from_vec(font_bytes).map_err(|_| TetraError::InvalidFont)?;

        Ok(VectorFontBuilder {
            data: VectorFontData::Owned(Rc::new(font)),
            texture_style: FontTextureStyle::Normal,
        })
    }

    /// Loads a vector font from a slice of binary data.
    ///
    /// # Errors
    ///
    /// * [`TetraError::InvalidFont`] will be returned if the font data was invalid.
    pub fn from_file_data(data: &'static [u8]) -> Result<VectorFontBuilder> {
        let font = FontRef::try_from_slice(data).map_err(|_| TetraError::InvalidFont)?;

        Ok(VectorFontBuilder {
            data: VectorFontData::Slice(Rc::new(font)),
            texture_style: FontTextureStyle::Normal,
        })
    }

    /// Sets which style of texture data should be generated for this font.
    pub fn texture_style(&mut self, texture_style: FontTextureStyle) -> &mut VectorFontBuilder {
        self.texture_style = texture_style;
        self
    }

    /// Creates a `Font` with the given size.
    ///
    /// # Errors
    ///
    /// * [`TetraError::PlatformError`] will be returned if the GPU cache for the font
    ///   could not be created.
    pub fn with_size(&self, ctx: &mut Context, size: f32) -> Result<Font> {
        let rasterizer: Box<dyn Rasterizer> = match &self.data {
            VectorFontData::Owned(f) => Box::new(VectorRasterizer::new(
                Rc::clone(f),
                size,
                self.texture_style,
            )),
            VectorFontData::Slice(f) => Box::new(VectorRasterizer::new(
                Rc::clone(f),
                size,
                self.texture_style,
            )),
        };

        let cache = FontCache::new(
            &mut ctx.device,
            rasterizer,
            ctx.graphics.default_filter_mode,
        )?;

        Ok(Font {
            data: Rc::new(RefCell::new(cache)),
        })
    }
}
