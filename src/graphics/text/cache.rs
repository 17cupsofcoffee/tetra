use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use unicode_normalization::UnicodeNormalization;

use crate::graphics::text::packer::ShelfPacker;
use crate::graphics::{FilterMode, Rectangle, Texture};
use crate::math::Vec2;
use crate::platform::GraphicsDevice;
use crate::{Context, Result};

/// The data produced by rasterizing a glyph from a font.
pub(crate) struct RasterizedGlyph {
    /// The rasterized RGBA data.
    pub data: Vec<u8>,

    /// How the glyph should be positioned, relative to the cursor's
    /// position on the baseline.
    ///
    /// The X and Y can be positive or negative, depending on the design of the font.
    /// The Y will almost always be negative though, as this is how the glyph is raised up
    /// to sit above the baseline.
    pub bounds: Rectangle,
}

/// The data produced by caching a glyph to the texture atlas.
struct CachedGlyph {
    /// The position of the glyph in the texture, in UV co-ordinates.
    uv: Rectangle,

    /// How the glyph should be positioned, relative to the cursor's
    /// position on the baseline.
    ///
    /// The X and Y can be positive or negative, depending on the design of the font.
    /// The Y will almost always be negative though, as this is how the glyph is raised up
    /// to sit above the baseline.
    bounds: Rectangle,
}

/// Errors that can occur when caching a glyph.
enum CacheError {
    /// Returned when the texture atlas is out of space.
    OutOfSpace,
}

/// A key identifying a glyph in the cache.
#[derive(PartialEq, Eq, Hash)]
struct CacheKey {
    /// The glyph's associated character.
    glyph: char,

    /// The glyph's horizontal subpixel offset (stored as a rounded integer).
    subpixel_x: u32,

    /// The glyph's vertical subpixel offset (stored as a rounded integer).
    subpixel_y: u32,
}

/// Implemented for types that can rasterize characters, and provide information
/// about their metrics.
pub(crate) trait Rasterizer {
    /// Rasterizes a character.
    ///
    /// The position may be taken into account if the font supports
    /// subpixel rendering.
    fn rasterize(&self, glyph: char, position: Vec2<f32>) -> Option<RasterizedGlyph>;

    /// The horizonal advance for a given glyph.
    fn advance(&self, glyph: char) -> f32;

    /// The height of the font.
    fn line_height(&self) -> f32;

    /// The ascent of the font.
    fn ascent(&self) -> f32;

    /// The amount of kerning that should be applied between the given glyphs.
    fn kerning(&self, previous: char, current: char) -> f32;
}

/// An individual quad within a `TextGeometry`.
#[derive(Debug, Clone)]
pub(crate) struct TextQuad {
    pub position: Rectangle,
    pub uv: Rectangle,
}

/// The geometry that can be used to render a piece of text.
#[derive(Debug, Clone)]
pub(crate) struct TextGeometry {
    pub quads: Vec<TextQuad>,
    pub bounds: Option<Rectangle>,
    pub resize_count: usize,
}

/// Renders text using a generated texture atlas.
pub(crate) struct FontCache {
    rasterizer: Box<dyn Rasterizer>,
    packer: ShelfPacker,
    glyphs: HashMap<CacheKey, Option<CachedGlyph>>,
    resize_count: usize,
}

impl FontCache {
    /// Creates a new cache, using the given rasterizer.
    pub fn new(
        device: &mut GraphicsDevice,
        rasterizer: Box<dyn Rasterizer>,
        filter_mode: FilterMode,
    ) -> Result<FontCache> {
        Ok(FontCache {
            rasterizer,
            packer: ShelfPacker::new(device, 128, 128, filter_mode)?,
            glyphs: HashMap::new(),
            resize_count: 0,
        })
    }

    /// Returns the current texture atlas.
    pub fn texture(&self) -> &Texture {
        self.packer.texture()
    }

    /// Returns the number of times that the cache has been resized.
    ///
    /// This can be compared against the `resize_count` of the `TextGeometry` to determine
    /// if that struct's data is stale.
    pub fn resize_count(&self) -> usize {
        self.resize_count
    }

    pub fn filter_mode(&self) -> FilterMode {
        self.packer.filter_mode()
    }

    pub fn set_filter_mode(&mut self, ctx: &mut Context, filter_mode: FilterMode) {
        self.packer.set_filter_mode(ctx, filter_mode);
    }

    /// Generates the geometry for the given string, resizing the texture atlas if needed.
    pub fn render(&mut self, device: &mut GraphicsDevice, input: &str) -> TextGeometry {
        loop {
            match self.try_render(device, input) {
                Ok(new_geometry) => return new_geometry,
                Err(CacheError::OutOfSpace) => {
                    self.resize(device).expect("Failed to resize font texture");
                }
            }
        }
    }

    /// Generates the geometry for the given string, returning an error if the texture atlas
    /// is out of space.
    fn try_render(
        &mut self,
        device: &mut GraphicsDevice,
        input: &str,
    ) -> std::result::Result<TextGeometry, CacheError> {
        let line_height = self.rasterizer.line_height().round();

        let mut quads = Vec::new();

        let mut cursor = Vec2::new(0.0, self.rasterizer.ascent().round());
        let mut last_glyph: Option<char> = None;
        let mut text_bounds: Option<Rectangle> = None;

        for ch in input.nfc() {
            if ch.is_control() {
                if ch == '\n' {
                    cursor.x = 0.0;
                    cursor.y += line_height;
                    last_glyph = None;
                }

                continue;
            }

            let subpixel_offset = cursor.map(f32::fract);

            // This is a bit of a hack to allow us to hash the subpixel offset:
            //
            // * Multiply by ten, so that the first decimal place becomes the integer part.
            // * Round to the closest number.
            //
            // So 0.05 becomes 0, 0.57 becomes 6, 0.99 becomes 10, etc. This effectively gives us
            // up to eleven different subpixel rendered versions of each glyph, which strikes
            // a nice balance between prettiness and reasonable texture size.
            //
            // We could wrap back around to 0 instead of 10 being a valid value, which would make
            // the distribution a bit more even, but I don't know if it's worth it.
            let subpixel_x = (subpixel_offset.x * 10.0).round() as u32;
            let subpixel_y = (subpixel_offset.y * 10.0).round() as u32;

            let cache_key = CacheKey {
                glyph: ch,
                subpixel_x,
                subpixel_y,
            };

            let cached_glyph = match self.glyphs.entry(cache_key) {
                Entry::Occupied(e) => e.into_mut(),
                Entry::Vacant(e) => {
                    let outline = match self.rasterizer.rasterize(ch, cursor) {
                        Some(r) => Some(add_glyph_to_texture(device, &mut self.packer, &r)?),
                        None => None,
                    };

                    e.insert(outline)
                }
            };

            if let Some(last_glyph) = last_glyph.take() {
                cursor.x += self.rasterizer.kerning(last_glyph, ch);
            }

            if let Some(CachedGlyph { mut bounds, uv }) = *cached_glyph {
                // The glyph's bounds are relative, so we need to combine them
                // with the cursor to make them absolute.
                bounds.x += cursor.x;
                bounds.y += cursor.y;

                // Expand the cached bounds of the text geometry:
                match &mut text_bounds {
                    Some(existing) => {
                        *existing = bounds.combine(existing);
                    }
                    None => {
                        text_bounds.replace(bounds);
                    }
                }

                quads.push(TextQuad {
                    position: bounds,
                    uv,
                });
            }

            cursor.x += self.rasterizer.advance(ch);

            last_glyph = Some(ch);
        }

        Ok(TextGeometry {
            quads,
            resize_count: self.resize_count,
            bounds: text_bounds,
        })
    }

    /// Resizes the texture atlas, clearing any cached data.
    fn resize(&mut self, device: &mut GraphicsDevice) -> Result {
        let (texture_width, texture_height) = self.packer.texture().size();

        let new_width = texture_width * 2;
        let new_height = texture_height * 2;

        self.packer.resize(device, new_width, new_height)?;
        self.glyphs.clear();

        self.resize_count += 1;

        Ok(())
    }
}

/// Adds a rasterized glyph to the texture atlas.
///
/// This is a free function rather than a method to avoid borrow checker issues.
fn add_glyph_to_texture(
    device: &mut GraphicsDevice,
    packer: &mut ShelfPacker,
    glyph: &RasterizedGlyph,
) -> std::result::Result<CachedGlyph, CacheError> {
    let (x, y) = packer
        .insert(
            device,
            &glyph.data,
            glyph.bounds.width as i32,
            glyph.bounds.height as i32,
        )
        .ok_or(CacheError::OutOfSpace)?;

    let (texture_width, texture_height) = packer.texture().size();

    Ok(CachedGlyph {
        bounds: glyph.bounds,
        uv: Rectangle::new(
            x as f32 / texture_width as f32,
            y as f32 / texture_height as f32,
            glyph.bounds.width / texture_width as f32,
            glyph.bounds.height / texture_height as f32,
        ),
    })
}
