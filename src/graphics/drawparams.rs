use crate::graphics::Color;
use crate::math::{Mat4, Vec2, Vec3};

/// Parameters that can be manipulated when drawing an object.
///
/// You can either use this as a builder by calling [`DrawParams::new`] and then chaining methods, or
/// construct it manually - whichever you find more pleasant to write.
#[derive(Debug, Clone, PartialEq)]
pub struct DrawParams {
    /// The position that the graphic should be drawn at. Defaults to `(0.0, 0.0)`.
    pub position: Vec2<f32>,

    /// The scale that the graphic should be drawn at. Defaults to `(1.0, 1.0)`.
    ///
    /// This can be set to a negative value to flip the graphic around the origin.
    pub scale: Vec2<f32>,

    /// The origin of the graphic. Defaults to `(0.0, 0.0)` (the top left).
    ///
    /// This offset is applied before scaling, rotation and positioning. For example, if you have
    /// a 16x16 image and set the origin to [8.0, 8.0], subsequent transformations will be performed
    /// relative to the center of the image.
    pub origin: Vec2<f32>,

    /// The rotation of the graphic, in radians. Defaults to `0.0`.
    pub rotation: f32,

    /// A color to multiply the graphic by. Defaults to [`Color::WHITE`].
    pub color: Color,
}

impl DrawParams {
    /// Creates a new set of `DrawParams`.
    pub fn new() -> DrawParams {
        DrawParams::default()
    }

    /// Sets the position that the graphic should be drawn at.
    pub fn position(mut self, position: Vec2<f32>) -> DrawParams {
        self.position = position;
        self
    }

    /// Sets the scale that the graphic should be drawn at.
    pub fn scale(mut self, scale: Vec2<f32>) -> DrawParams {
        self.scale = scale;
        self
    }

    /// Sets the origin of the graphic.
    pub fn origin(mut self, origin: Vec2<f32>) -> DrawParams {
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

    /// Creates a new transformation matrix equivalent to this set of params.
    ///
    /// This method does not take into account `color`, as it cannot
    /// be represented via a matrix.
    pub fn to_matrix(&self) -> Mat4<f32> {
        let mut matrix = Mat4::translation_2d(-self.origin);
        matrix.scale_3d(Vec3::from(self.scale));
        matrix.rotate_z(self.rotation);
        matrix.translate_2d(self.position);
        matrix
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
        }
    }
}

impl From<Vec2<f32>> for DrawParams {
    fn from(position: Vec2<f32>) -> DrawParams {
        DrawParams {
            position,
            ..DrawParams::default()
        }
    }
}

impl From<DrawParams> for Mat4<f32> {
    fn from(params: DrawParams) -> Self {
        params.to_matrix()
    }
}
