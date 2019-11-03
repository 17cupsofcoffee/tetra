use crate::graphics::{Color, Rectangle};
use crate::math::Vec2;
use crate::Context;

/// Struct representing the parameters that can be used when drawing.
///
/// You can either use this as a builder by calling `DrawParams::new` and then chaining methods, or
/// construct it manually - whichever you find more pleasant to write.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawParams {
    /// The position that the graphic should be drawn at. Defaults to [0.0, 0.0].
    pub position: Vec2,

    /// The scale that the graphic should be drawn at. Defaults to [1.0, 1.0].
    ///
    /// This can be set to a negative value to flip the graphic around the origin.
    pub scale: Vec2,

    /// The origin of the graphic. Defaults to [0.0, 0.0] (the top left).
    ///
    /// Positioning and scaling will be calculated relative to this point.
    pub origin: Vec2,

    /// The rotation of the graphic, in radians. Defaults to 0.0.
    pub rotation: f32,

    /// A color to multiply the graphic by. Defaults to white.
    pub color: Color,

    /// A sub-region of the graphic to draw. Defaults to `None`, which means the the full graphic will be drawn.
    ///
    /// This is useful if you're using spritesheets (which you should be!).
    pub clip: Option<Rectangle>,
}

impl DrawParams {
    /// Creates a new set of `DrawParams`.
    pub fn new() -> DrawParams {
        DrawParams::default()
    }

    /// Sets the position that the graphic should be drawn at.
    pub fn position(mut self, position: Vec2) -> DrawParams {
        self.position = position;
        self
    }

    /// Sets the scale that the graphic should be drawn at.
    pub fn scale(mut self, scale: Vec2) -> DrawParams {
        self.scale = scale;
        self
    }

    /// Sets the origin of the graphic.
    pub fn origin(mut self, origin: Vec2) -> DrawParams {
        self.origin = origin;
        self
    }

    /// Sets the rotation of the graphic, in radians.
    pub fn rotation(mut self, rotation: f32) -> DrawParams {
        self.rotation = rotation;
        self
    }

    /// Sets the color to multiply the graphic by.
    pub fn color(mut self, color: Color) -> DrawParams {
        self.color = color;
        self
    }

    /// Sets the region of the graphic to draw.
    pub fn clip(mut self, clip: Rectangle) -> DrawParams {
        self.clip = Some(clip);
        self
    }
}

impl Default for DrawParams {
    fn default() -> DrawParams {
        DrawParams {
            position: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0),
            origin: Vec2::new(0.0, 0.0),
            rotation: 0.0,
            color: Color::WHITE,
            clip: None,
        }
    }
}

impl From<Vec2> for DrawParams {
    fn from(position: Vec2) -> DrawParams {
        DrawParams {
            position,
            ..DrawParams::default()
        }
    }
}

/// Represents a type that can be drawn.
///
/// [`graphics::draw`](fn.draw.html) can be used to draw without importing this trait, which is sometimes
/// more convienent.
pub trait Drawable {
    /// Draws `self` to the screen (or a canvas, if one is enabled), using the specified parameters.
    ///
    /// Any type that implements `Into<DrawParams>` can be passed into this method. For example, since the majority
    /// of the time, you only care about changing the position, a `Vec2` can be passed to set the position and leave
    /// everything else as the defaults.
    fn draw<P>(&self, ctx: &mut Context, params: P)
    where
        P: Into<DrawParams>;
}
