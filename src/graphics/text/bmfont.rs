use std::path::Path;
use std::str::FromStr;

use hashbrown::HashMap;
use image::{EncodableLayout, RgbaImage, SubImage};

use crate::fs;
use crate::graphics::text::cache::{RasterizedGlyph, Rasterizer};
use crate::graphics::Rectangle;
use crate::math::Vec2;
use crate::{Result, TetraError};

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

pub struct BMFontRasterizer {
    line_height: u32,
    base: u32,

    pages: HashMap<u32, RgbaImage>,
    glyphs: HashMap<u32, BMFontGlyph>,
    kerning: HashMap<(u32, u32), i32>,
}

impl BMFontRasterizer {
    pub fn new<P>(path: P) -> Result<BMFontRasterizer>
    where
        P: AsRef<Path>,
    {
        let path = path.as_ref();

        let bmfont = fs::read_to_string(path)?;

        let mut line_height = None;
        let mut base = None;
        let mut pages = HashMap::new();
        let mut glyphs = HashMap::new();
        let mut kerning = HashMap::new();

        for line in bmfont.lines() {
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
                    let file = attributes.get("file")?;
                    let file_path = path.parent().ok_or(TetraError::InvalidFont)?.join(file);

                    let image = fs::read_to_image(&file_path)?.to_rgba8();

                    pages.insert(id, image);
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
