use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use hashbrown::HashMap;

use crate::graphics::text::cache::{RasterizedGlyph, Rasterizer};
use crate::graphics::{ImageData, Rectangle, TextureFormat};
use crate::math::Vec2;
use crate::{fs, Context};
use crate::{Result, TetraError};

use super::cache::FontCache;
use super::Font;

struct BmFontGlyph {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    x_offset: i32,
    y_offset: i32,
    x_advance: i32,
    page: u32,
}

/// A builder for fonts stored in the AngelCode BMFont format.
///
/// Currently, only the text format is supported. Support for the binary file
/// format may be added in the future.
///
/// [`Font::bmfont`] provides a simpler API for loading vector fonts, if you don't need
/// all of the functionality of this struct.
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
/// # Performance
///
/// Creating or cloning a `BmFontBuilder` can be expensive, depending on how many images you
/// load into it.
///
/// The data is not internally reference-counted (unlike some other Tetra structs like
/// [`VectorFontBuilder`](super::VectorFontBuilder)), as you'll generally only use a
/// `BmFontBuilder` once. This allows the builder's buffers to be re-used by the
/// created [`Font`].
#[derive(Debug, Clone)]
pub struct BmFontBuilder {
    font: String,
    image_dir: Option<PathBuf>,
    pages: HashMap<u32, ImageData>,
}

impl BmFontBuilder {
    /// Loads a BMFont from the given file.
    ///
    /// By default, the image directory will be set to the same directory as the
    /// font itself.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if the file could not be loaded.
    pub fn new<P>(path: P) -> Result<BmFontBuilder>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let font = fs::read_to_string(path)?;

        // This should be okay to unwrap, if the font itself loaded...
        let image_dir = path.parent().unwrap().to_owned();

        Ok(BmFontBuilder {
            font,
            image_dir: Some(image_dir),
            pages: HashMap::new(),
        })
    }

    /// Loads a BMFont from a string.
    ///
    /// As a BMFont only contains relative paths, you will need to specify an image
    /// directory and/or page data in order for the font to successfully build.
    pub fn from_file_data<D>(data: D) -> BmFontBuilder
    where
        D: Into<String>,
    {
        BmFontBuilder {
            font: data.into(),
            image_dir: None,
            pages: HashMap::new(),
        }
    }

    /// Sets the directory to search for the font's image files.
    ///
    /// This will automatically be set if the builder was created via [`new`](Self::new),
    /// but can be overridden.
    ///
    /// If all of the font's pages are manually loaded via the other builder methods,
    /// this path will be ignored.
    pub fn with_image_dir<P>(mut self, path: P) -> BmFontBuilder
    where
        P: Into<PathBuf>,
    {
        self.image_dir = Some(path.into());
        self
    }

    /// Loads an image for the specified page of the font.
    ///
    /// This will override the path specified in the font itself.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if a file could not be loaded.
    /// * [`TetraError::InvalidTexture`] will be returned if some of the image data was invalid.
    pub fn with_page<P>(mut self, id: u32, path: P) -> Result<BmFontBuilder>
    where
        P: AsRef<Path>,
    {
        self.pages.insert(id, ImageData::new(path)?);

        Ok(self)
    }

    /// Sets the image for the specified page of the font, using raw pixel data.
    ///
    /// This will override the path specified in the font itself.
    ///
    /// # Errors
    ///
    /// * [`TetraError::NotEnoughData`] will be returned if not enough data is provided to fill
    ///   the texture.
    pub fn with_page_data<D>(
        mut self,
        id: u32,
        width: i32,
        height: i32,
        format: TextureFormat,
        data: D,
    ) -> Result<BmFontBuilder>
    where
        D: Into<Vec<u8>>,
    {
        self.pages
            .insert(id, ImageData::from_data(width, height, format, data)?);

        Ok(self)
    }

    /// Sets the image for the specified page of the font, using data encoded in
    /// one of Tetra's supported image file formats.
    ///
    /// The format of the data will be determined based on the 'magic bytes' at the
    /// beginning of the data. Note that TGA files do not have recognizable magic
    /// bytes, so this function will not recognize them.
    ///
    /// This will override the path specified in the font itself.
    ///
    /// # Errors
    ///
    /// * [`TetraError::InvalidTexture`] will be returned if the image data was invalid.
    pub fn with_page_encoded(mut self, id: u32, data: &[u8]) -> Result<BmFontBuilder> {
        self.pages.insert(id, ImageData::from_encoded(data)?);

        Ok(self)
    }

    /// Sets the image for the specified page of the font, using decoded image data.
    ///
    /// This will override the path specified in the font itself.
    pub fn with_page_image_data(mut self, id: u32, data: ImageData) -> BmFontBuilder {
        self.pages.insert(id, data);

        self
    }

    /// Builds the font.
    ///
    /// Any pages that have not had their images manually set will be loaded from the path
    /// specified by [`with_image_dir`](Self::with_image_dir).
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if a file could not be loaded.
    /// * [`TetraError::InvalidTexture`] will be returned if some of the image data was invalid.
    /// * [`TetraError::InvalidFont`] will be returned if the font definition was invalid,
    ///   or if there was no path specified for one of the image files.
    /// * [`TetraError::PlatformError`] will be returned if the GPU cache for the font
    ///   could not be created.
    pub fn build(self, ctx: &mut Context) -> Result<Font> {
        let rasterizer: Box<dyn Rasterizer> = Box::new(BmFontRasterizer::new(
            &self.font,
            self.image_dir,
            self.pages,
        )?);

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

pub struct BmFontRasterizer {
    line_height: u32,
    base: u32,

    pages: HashMap<u32, ImageData>,
    glyphs: HashMap<u32, BmFontGlyph>,
    kerning: HashMap<(u32, u32), i32>,
}

impl BmFontRasterizer {
    fn new(
        font: &str,
        image_path: Option<PathBuf>,
        mut pages: HashMap<u32, ImageData>,
    ) -> Result<BmFontRasterizer> {
        let mut line_height = None;
        let mut base = None;
        let mut glyphs = HashMap::new();
        let mut kerning = HashMap::new();

        for line in font.lines() {
            let (tag, attributes) = parse_tag(line);

            match tag {
                "common" => {
                    let attributes = parse_attributes(attributes)?;

                    line_height = Some(attributes.parse("lineHeight")?);
                    base = Some(attributes.parse("base")?);
                }

                "page" => {
                    let attributes = parse_attributes(attributes)?;

                    let id = attributes.parse("id")?;

                    if !pages.contains_key(&id) {
                        let file = attributes.get("file")?;

                        let file_path = image_path
                            .as_ref()
                            .ok_or(TetraError::InvalidFont)?
                            .join(file);

                        pages.insert(id, ImageData::new(file_path)?);
                    }
                }

                "char" => {
                    let attributes = parse_attributes(attributes)?;

                    let id = attributes.parse("id")?;

                    let glyph = BmFontGlyph {
                        x: attributes.parse("x")?,
                        y: attributes.parse("y")?,
                        width: attributes.parse("width")?,
                        height: attributes.parse("height")?,
                        x_offset: attributes.parse("xoffset")?,
                        y_offset: attributes.parse("yoffset")?,
                        x_advance: attributes.parse("xadvance")?,
                        page: attributes.parse("page")?,
                    };

                    glyphs.insert(id, glyph);
                }

                "kerning" => {
                    let attributes = parse_attributes(attributes)?;

                    let first = attributes.parse("first")?;
                    let second = attributes.parse("second")?;
                    let amount = attributes.parse("amount")?;

                    kerning.insert((first, second), amount);
                }

                _ => {}
            }
        }

        Ok(BmFontRasterizer {
            line_height: line_height.ok_or(TetraError::InvalidFont)?,
            base: base.ok_or(TetraError::InvalidFont)?,
            pages,
            glyphs,
            kerning,
        })
    }
}

impl Rasterizer for BmFontRasterizer {
    fn rasterize(&self, glyph: char, _: Vec2<f32>) -> Option<RasterizedGlyph> {
        if let Some(bmglyph) = self.glyphs.get(&(glyph as u32)) {
            let page = self.pages.get(&bmglyph.page)?;

            let region = page.region(Rectangle::new(
                bmglyph.x as i32,
                bmglyph.y as i32,
                bmglyph.width as i32,
                bmglyph.height as i32,
            ));

            Some(RasterizedGlyph {
                data: region.as_bytes().into(),
                bounds: Rectangle::new(
                    bmglyph.x_offset as f32,
                    // This is done for consistency with the TTF rasterizer,
                    // which measures offsets relative to the cursor rather than
                    // the top of the cell:
                    -(self.base as i32 - bmglyph.y_offset) as f32,
                    bmglyph.width as f32,
                    bmglyph.height as f32,
                ),
            })
        } else {
            None
        }
    }

    fn advance(&self, glyph: char) -> f32 {
        self.glyphs
            .get(&(glyph as u32))
            .map(|bmchar| bmchar.x_advance as f32)
            .unwrap_or(0.0)
    }

    fn line_height(&self) -> f32 {
        self.line_height as f32
    }

    fn ascent(&self) -> f32 {
        self.base as f32
    }

    fn kerning(&self, previous: char, current: char) -> f32 {
        self.kerning
            .get(&(previous as u32, current as u32))
            .copied()
            .unwrap_or(0) as f32
    }
}

struct BmFontAttributes<'a> {
    attributes: HashMap<&'a str, &'a str>,
}

impl BmFontAttributes<'_> {
    fn get(&self, key: &str) -> Result<&str> {
        self.attributes
            .get(key)
            .copied()
            .ok_or(TetraError::InvalidFont)
    }

    fn parse<T>(&self, key: &str) -> Result<T>
    where
        T: FromStr,
    {
        let value = self.get(key)?;
        value.parse().map_err(|_| TetraError::InvalidFont)
    }
}

fn parse_tag(input: &str) -> (&str, &str) {
    let trimmed = input.trim_start();
    let tag_end = trimmed.find(' ').unwrap_or_else(|| trimmed.len());
    trimmed.split_at(tag_end)
}

fn parse_attributes(input: &str) -> Result<BmFontAttributes<'_>> {
    let mut remaining = input.trim_start();
    let mut attributes = HashMap::new();

    while !remaining.is_empty() {
        // Find the next key by looking for a '='.
        let key_end = remaining.find('=').ok_or(TetraError::InvalidFont)?;
        let (key, next) = remaining.split_at(key_end);

        // Skip past the '='.
        remaining = &next[1..];

        // Is the value a string?
        if remaining.starts_with('"') {
            // Skip past the '"'.
            remaining = &remaining[1..];

            // Find the end of the value by searching for a closing '"'.
            let value_end = remaining.find('"').ok_or(TetraError::InvalidFont)?;
            let (value, next) = remaining.split_at(value_end);

            attributes.insert(key, value);

            // Skip past the closing '"', and any trailing whitespace.
            remaining = next[1..].trim_start();
        } else {
            // Find the end of the value by searching for whitespace.
            // If we don't find it, this must be the end of the line.
            let value_end = remaining.find(' ').unwrap_or_else(|| remaining.len());
            let (value, next) = remaining.split_at(value_end);

            attributes.insert(key, value);

            // Skip past any trailing whitespace.
            remaining = next.trim_start();
        }
    }

    Ok(BmFontAttributes { attributes })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_valid_line() {
        let (tag, rest) = parse_tag("tag keyA=123 keyB=\"string\" keyC=1,2,3,4");

        assert_eq!("tag", tag);
        assert_eq!(" keyA=123 keyB=\"string\" keyC=1,2,3,4", rest);

        let attributes = parse_attributes(rest).unwrap();

        assert_eq!(attributes.get("keyA").unwrap(), "123");
        assert_eq!(attributes.get("keyB").unwrap(), "string");
        assert_eq!(attributes.get("keyC").unwrap(), "1,2,3,4");
    }

    #[test]
    fn parse_valid_line_with_whitespace() {
        let (tag, rest) = parse_tag("   tag    keyA=123   ");

        assert_eq!("tag", tag);
        assert_eq!("    keyA=123   ", rest);

        let attributes = parse_attributes(rest).unwrap();

        assert_eq!(attributes.get("keyA").unwrap(), "123");
    }

    #[test]
    fn parse_valid_line_with_no_data() {
        let (tag, rest) = parse_tag("   tag");

        assert_eq!("tag", tag);
        assert_eq!("", rest);

        let attributes = parse_attributes(rest).unwrap();
        assert!(attributes.attributes.is_empty());
    }

    #[test]
    #[should_panic]
    fn parse_invalid_line_with_missing_equals() {
        let (tag, rest) = parse_tag("   tag keyA");

        assert_eq!("tag", tag);
        assert_eq!(" keyA", rest);

        parse_attributes(rest).unwrap();
    }

    #[test]
    #[should_panic]
    fn parse_invalid_line_with_missing_string_terminator() {
        let (tag, rest) = parse_tag("   tag keyA=\"string");

        assert_eq!("tag", tag);
        assert_eq!(" keyA=\"string", rest);

        parse_attributes(rest).unwrap();
    }
}
