use std::rc::Rc;

use ab_glyph::{Font as AbFont, PxScale, ScaleFont};

use crate::graphics::text::cache::{RasterizedGlyph, Rasterizer};
use crate::graphics::Rectangle;
use crate::math::Vec2;

pub(crate) struct VectorRasterizer<F> {
    font: Rc<F>,
    scale: PxScale,
}

impl<F> VectorRasterizer<F>
where
    F: AbFont,
{
    pub fn new(font: Rc<F>, size: f32) -> VectorRasterizer<F> {
        VectorRasterizer {
            font,
            scale: PxScale::from(size),
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
                data.extend_from_slice(&[255, 255, 255, (v * 255.0) as u8]);
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

    fn h_advance(&self, glyph: char) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.h_advance(scaled_font.glyph_id(glyph))
    }

    fn height(&self) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.height()
    }

    fn line_gap(&self) -> f32 {
        let scaled_font = self.font.as_scaled(self.scale);

        scaled_font.line_gap()
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
