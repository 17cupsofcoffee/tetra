use hashbrown::hash_map::Entry;
use hashbrown::HashMap;
use xi_unicode::LineBreakIterator;

use crate::graphics::text::packer::ShelfPacker;
use crate::graphics::{FilterMode, Rectangle, Texture};
use crate::math::Vec2;
use crate::platform::GraphicsDevice;
use crate::{Context, Result};

/// The data produced by rasterizing a glyph from a font.
pub(crate) struct RasterizedGlyph {
    /// The bounds of the glyph.
    ///
    /// The X and Y are relative to the cursor's position on the baseline, and can be
    /// positive or negative, depending on the design of the font. The Y will almost
    /// always be negative, as this is how the glyph is raised up to sit above the
    /// baseline.
    pub bounds: Rectangle,

    /// The rasterized RGBA data.
    pub data: Vec<u8>,
}

/// An individual quad within a `TextGeometry`.
#[derive(Debug, Copy, Clone)]
pub struct TextQuad {
    /// The position of the glyph, relative to the text's origin.
    pub position: Vec2<f32>,

    /// The location of the glyph in the font's texture.
    pub region: Rectangle,
}

impl TextQuad {
    fn bounds(&self) -> Rectangle {
        Rectangle::new(
            self.position.x,
            self.position.y,
            self.region.width,
            self.region.height,
        )
    }
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
    glyphs: HashMap<CacheKey, Option<TextQuad>>,
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
    pub fn render(
        &mut self,
        device: &mut GraphicsDevice,
        input: &str,
        max_width: Option<f32>,
    ) -> TextGeometry {
        loop {
            match self.try_render(device, input, max_width) {
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
        max_width: Option<f32>,
    ) -> std::result::Result<TextGeometry, CacheError> {
        let line_height = self.rasterizer.line_height().round();

        let mut quads = Vec::new();

        let mut cursor = Vec2::new(0.0, self.rasterizer.ascent().round());
        let mut last_glyph: Option<char> = None;
        let mut text_bounds: Option<Rectangle> = None;
        let mut words_on_line = 0;

        for (word, _) in UnicodeLineBreaks::new(input) {
            if let Some(max_width) = max_width {
                // We only allow wrapping to take place after the first word on each line,
                // to avoid extra line breaks appearing when a word is too long to fit on
                // a single line.
                if words_on_line > 0 && cursor.x + self.measure_word(word) > max_width {
                    cursor.x = 0.0;
                    cursor.y += line_height;
                    last_glyph = None;
                    words_on_line = 0;
                }
            }

            words_on_line += 1;

            for ch in word.chars() {
                if ch.is_control() {
                    if ch == '\n' {
                        cursor.x = 0.0;
                        cursor.y += line_height;
                        last_glyph = None;
                        words_on_line = 0;
                    }

                    continue;
                }

                if let Some(last_glyph) = last_glyph {
                    cursor.x += self.rasterizer.kerning(last_glyph, ch);
                }

                if let Some(quad) = self.rasterize_char(device, ch, cursor)? {
                    // Expand the cached bounds of the text geometry:
                    match &mut text_bounds {
                        Some(existing) => *existing = quad.bounds().combine(existing),
                        None => {
                            text_bounds.replace(quad.bounds());
                        }
                    }

                    quads.push(quad);
                }

                cursor.x += self.rasterizer.advance(ch);

                last_glyph = Some(ch);
            }
        }

        Ok(TextGeometry {
            quads,
            resize_count: self.resize_count,
            bounds: text_bounds,
        })
    }

    /// Measures the width of a word, not including any trailing whitespace.
    ///
    /// This is mainly used to determine if a word needs to break onto a
    /// new line.
    fn measure_word(&self, word: &str) -> f32 {
        let mut last_glyph = None;
        let mut word_width = 0.0;

        for ch in word.trim_end().chars() {
            word_width += self.rasterizer.advance(ch);

            if let Some(last) = last_glyph {
                word_width += self.rasterizer.kerning(last, ch);
            }

            last_glyph = Some(ch);
        }

        word_width
    }

    /// Rasterizes a character with a given position, or pull it from the texture cache.
    fn rasterize_char(
        &mut self,
        device: &mut GraphicsDevice,
        ch: char,
        position: Vec2<f32>,
    ) -> std::result::Result<Option<TextQuad>, CacheError> {
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
        let subpixel_offset = position.map(f32::fract);
        let subpixel_x = (subpixel_offset.x * 10.0).round() as u32;
        let subpixel_y = (subpixel_offset.y * 10.0).round() as u32;

        let cache_key = CacheKey {
            glyph: ch,
            subpixel_x,
            subpixel_y,
        };

        let cached_quad = match self.glyphs.entry(cache_key) {
            Entry::Occupied(e) => e.into_mut(),
            Entry::Vacant(e) => {
                let outline = match self.rasterizer.rasterize(ch, position) {
                    Some(r) => Some(add_glyph_to_texture(device, &mut self.packer, &r)?),
                    None => None,
                };

                e.insert(outline)
            }
        };

        if let Some(mut quad) = *cached_quad {
            // The cached quad's bounds are relative, so we need to combine them
            // with the position to make them absolute.
            quad.position += position;

            Ok(Some(quad))
        } else {
            Ok(None)
        }
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
) -> std::result::Result<TextQuad, CacheError> {
    const PADDING: i32 = 1;

    let region = packer
        .insert(
            device,
            &glyph.data,
            glyph.bounds.width as i32,
            glyph.bounds.height as i32,
            PADDING,
        )
        .ok_or(CacheError::OutOfSpace)?;

    Ok(TextQuad {
        position: Vec2::new(
            glyph.bounds.x - PADDING as f32,
            glyph.bounds.y - PADDING as f32,
        ),
        region: Rectangle::new(
            region.x as f32,
            region.y as f32,
            region.width as f32,
            region.height as f32,
        ),
    })
}

struct UnicodeLineBreaks<'a> {
    input: &'a str,
    breaker: LineBreakIterator<'a>,
    last_break: usize,
}

impl<'a> UnicodeLineBreaks<'a> {
    fn new(input: &'a str) -> UnicodeLineBreaks<'a> {
        UnicodeLineBreaks {
            input,
            breaker: LineBreakIterator::new(input),
            last_break: 0,
        }
    }
}

impl<'a> Iterator for UnicodeLineBreaks<'a> {
    type Item = (&'a str, bool);

    fn next(&mut self) -> Option<Self::Item> {
        self.breaker.next().map(|(offset, hard_break)| {
            let word = &self.input[self.last_break..offset];
            self.last_break = offset;
            (word, hard_break)
        })
    }
}
