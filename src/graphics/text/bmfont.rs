use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use hashbrown::HashMap;
use image::{EncodableLayout, RgbaImage, SubImage};

use crate::graphics::text::cache::{RasterizedGlyph, Rasterizer};
use crate::graphics::Rectangle;
use crate::math::Vec2;
use crate::{fs, Context};
use crate::{Result, TetraError};

use super::cache::FontCache;
use super::Font;

struct BMFontGlyph {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    x_offset: i32,
    y_offset: i32,
    x_advance: i32,
    page: u32,
}

enum PageData {
    Path(PathBuf),
    FileData(&'static [u8]),
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
/// Creating a `BMFontBuilder` is a relatively expensive operation, but you will usually only
/// need to do it once, when loading your font for the first time.
pub struct BMFontBuilder {
    font: String,
    image_dir: Option<PathBuf>,
    pages: HashMap<u32, PageData>,
}

impl BMFontBuilder {
    /// Loads a BMFont from the given file.
    ///
    /// By default, the image directory will be set to the same directory as the
    /// font itself.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if the file could not be loaded.
    pub fn new<P>(path: P) -> Result<BMFontBuilder>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();
        let font = fs::read_to_string(path)?;

        // This should be okay to unwrap, if the font itself loaded...
        let image_dir = path.parent().unwrap().to_owned();

        Ok(BMFontBuilder {
            font,
            image_dir: Some(image_dir),
            pages: HashMap::new(),
        })
    }

    /// Loads a BMFont from a string.
    ///
    /// As a BMFont only contains relative paths, you will need to specify an image
    /// directory (via [`with_image_dir`](Self::with_image_dir)) and/or page data
    /// (via [`with_page`](Self::with_page) and [`with_page_file_data`](Self::with_page_file_data))
    /// in order for the font to successfully build.
    pub fn from_file_data<D>(data: D) -> BMFontBuilder
    where
        D: Into<String>,
    {
        BMFontBuilder {
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
    /// If all of the font's pages are manually specified via
    /// [`with_page`](Self::with_page) and/or [`with_page_file_data`](Self::with_page_file_data),
    /// this will be ignored.
    pub fn with_image_dir<P>(&mut self, path: P) -> &mut BMFontBuilder
    where
        P: Into<PathBuf>,
    {
        self.image_dir = Some(path.into());
        self
    }

    /// Sets the image path for the specified page of the font.
    ///
    /// This will override the path specified in the font itself.
    pub fn with_page<P>(&mut self, id: u32, path: P) -> &mut BMFontBuilder
    where
        P: Into<PathBuf>,
    {
        self.pages.insert(id, PageData::Path(path.into()));
        self
    }

    /// Sets the image data for the specified page of the font.
    ///
    /// The data must be encoded in one of Tetra's supported image file formats.
    /// The format of the data will be determined based on the 'magic bytes' at the
    /// beginning of the data. Note that TGA files do not have recognizable magic
    /// bytes, so this function will not recognize them.
    ///
    /// This will override the path specified in the font itself.
    pub fn with_page_file_data(&mut self, id: u32, data: &'static [u8]) -> &mut BMFontBuilder {
        self.pages.insert(id, PageData::FileData(data));
        self
    }

    /// Builds the font.
    ///
    /// # Errors
    ///
    /// * [`TetraError::FailedToLoadAsset`] will be returned if a file could not be loaded.
    /// * [`TetraError::InvalidTexture`] will be returned if some of the image data was invalid.
    /// * [`TetraError::InvalidFont`] will be returned if the font definition was invalid,
    ///   or if there was no path specified for one of the image files.
    /// * [`TetraError::PlatformError`] will be returned if the GPU cache for the font
    ///   could not be created.
    pub fn build(&mut self, ctx: &mut Context) -> Result<Font> {
        let rasterizer: Box<dyn Rasterizer> = Box::new(BMFontRasterizer::new(
            &self.font,
            self.image_dir.as_ref(),
            &self.pages,
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

pub struct BMFontRasterizer {
    line_height: u32,
    base: u32,

    pages: HashMap<u32, RgbaImage>,
    glyphs: HashMap<u32, BMFontGlyph>,
    kerning: HashMap<(u32, u32), i32>,
}

impl BMFontRasterizer {
    fn new(
        font: &str,
        image_path: Option<&PathBuf>,
        page_data: &HashMap<u32, PageData>,
    ) -> Result<BMFontRasterizer> {
        let mut line_height = None;
        let mut base = None;
        let mut pages = HashMap::new();
        let mut glyphs = HashMap::new();
        let mut kerning = HashMap::new();

        for (&id, page) in page_data {
            let image = match page {
                PageData::Path(p) => fs::read_to_image(p)?.to_rgba8(),
                PageData::FileData(d) => image::load_from_memory(d)
                    .map_err(TetraError::InvalidTexture)?
                    .to_rgba8(),
            };

            pages.insert(id, image);
        }

        for line in font.lines() {
            let (tag, attributes) = parse_tag(line)?;

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
                        let file_path = image_path.ok_or(TetraError::InvalidFont)?.join(file);

                        let image = fs::read_to_image(&file_path)?.to_rgba8();

                        pages.insert(id, image);
                    }
                }

                "char" => {
                    let attributes = parse_attributes(attributes)?;

                    let id = attributes.parse("id")?;

                    let glyph = BMFontGlyph {
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

        Ok(BMFontRasterizer {
            line_height: line_height.ok_or(TetraError::InvalidFont)?,
            base: base.ok_or(TetraError::InvalidFont)?,
            pages,
            glyphs,
            kerning,
        })
    }
}

impl Rasterizer for BMFontRasterizer {
    fn rasterize(&self, glyph: char, _: Vec2<f32>) -> Option<RasterizedGlyph> {
        if let Some(bmglyph) = self.glyphs.get(&(glyph as u32)) {
            let page = self.pages.get(&bmglyph.page)?;

            let subimage = SubImage::new(page, bmglyph.x, bmglyph.y, bmglyph.width, bmglyph.height);

            let buffer = subimage.to_image();

            Some(RasterizedGlyph {
                data: buffer.as_bytes().into(),
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

struct BMFontAttributes<'a> {
    attributes: HashMap<&'a str, &'a str>,
}

impl BMFontAttributes<'_> {
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

fn parse_tag(input: &str) -> Result<(&str, &str)> {
    let trimmed = input.trim_start();
    let tag_end = trimmed.find(' ').ok_or(TetraError::InvalidFont)?;
    Ok(trimmed.split_at(tag_end))
}

fn parse_attributes(input: &str) -> Result<BMFontAttributes<'_>> {
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
            remaining = &next[1..].trim_start();
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

    Ok(BMFontAttributes { attributes })
}
