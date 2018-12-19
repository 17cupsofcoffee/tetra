//! Functions and types relating to color.

/// Represents an RGBA color.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Color {
    /// The red component of the color.
    pub r: f32,

    /// The green component of the color.
    pub g: f32,

    /// The blue component of the color.
    pub b: f32,

    /// The alpha component of the color.
    pub a: f32,
}

impl Color {
    /// Creates a new `Color`, with the specified RGB values and the alpha set to 1.0.
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    /// Creates a new `Color`, with the specified RGBA values.
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

/// Shortcut for Color::rgb(0.0, 0.0, 0.0).
pub const BLACK: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 1.0,
};

/// Shortcut for Color::rgb(1.0, 1.0, 1.0).
pub const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};
