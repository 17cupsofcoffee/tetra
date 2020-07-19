//! Functions and types relating to color.

use crate::error::{Result, TetraError};

/// An RGBA color.
///
/// The components are stored as `f32` values in the range of 0.0 to 1.0.
/// If your data is made up of bytes or hex values, this type provides
/// constructors that will carry out the conversion for you.
///
/// # Serde
///
/// Serialization and deserialization of this type (via [Serde](https://serde.rs/))
/// can be enabled via the `serde_support` feature.
#[derive(Debug, Copy, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde_support",
    derive(serde::Serialize, serde::Deserialize)
)]
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
    pub const fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    /// Creates a new `Color`, with the specified RGBA values.
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }

    /// Creates a new `Color`, with the specified RGB integer (0-255) values and the alpha set to 255.
    pub fn rgb8(r: u8, g: u8, b: u8) -> Color {
        let r = f32::from(r) / 255.0;
        let g = f32::from(g) / 255.0;
        let b = f32::from(b) / 255.0;

        Color { r, g, b, a: 1.0 }
    }

    /// Creates a new `Color`, with the specified RGBA (0-255) integer values.
    pub fn rgba8(r: u8, g: u8, b: u8, a: u8) -> Color {
        let r = f32::from(r) / 255.0;
        let g = f32::from(g) / 255.0;
        let b = f32::from(b) / 255.0;
        let a = f32::from(a) / 255.0;

        Color { r, g, b, a }
    }

    /// Creates a new `Color` using a hexidecimal color code, panicking if the input is
    /// invalid.
    ///
    /// Six and eight digit codes can be used - the former will be interpreted as RGB, and
    /// the latter as RGBA. The `#` prefix (commonly used on the web) will be stripped if present.
    pub fn hex(hex: &str) -> Color {
        let hex = hex.trim_start_matches('#');

        assert!(hex.len() == 6 || hex.len() == 8);

        let r = u8::from_str_radix(&hex[0..2], 16).unwrap();
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap();
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap();

        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).unwrap()
        } else {
            255
        };

        Color::rgba8(r, g, b, a)
    }

    /// Creates a new `Color` using a hexidecimal color code, returning an error if the
    /// input is invalid.
    ///
    /// Six and eight digit codes can be used - the former will be interpreted as RGB, and
    /// the latter as RGBA. The `#` prefix (commonly used on the web) will be stripped if present.
    ///
    /// # Errors
    ///
    /// * `TetraError::InvalidColor` will be returned if the specified color is invalid.
    pub fn try_hex(hex: &str) -> Result<Color> {
        let hex = hex.trim_start_matches('#');

        if hex.len() != 6 && hex.len() != 8 {
            return Err(TetraError::InvalidColor);
        }

        let r = u8::from_str_radix(&hex[0..2], 16).map_err(|_| TetraError::InvalidColor)?;
        let g = u8::from_str_radix(&hex[2..4], 16).map_err(|_| TetraError::InvalidColor)?;
        let b = u8::from_str_radix(&hex[4..6], 16).map_err(|_| TetraError::InvalidColor)?;

        let a = if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).map_err(|_| TetraError::InvalidColor)?
        } else {
            255
        };

        Ok(Color::rgba8(r, g, b, a))
    }

    // These constants should remain at the bottom of the impl block to keep
    // the docs readable - don't want to have to scroll through a load of colors
    // to get to the methods!

    /// Shortcut for `Color::rgb(0.0, 0.0, 0.0)`.
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);

    /// Shortcut for `Color::rgb(1.0, 1.0, 1.0)`.
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    /// Shortcut for `Color::rgb(1.0, 0.0, 0.0)`.
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);

    /// Shortcut for `Color::rgb(0.0, 1.0, 0.0)`.
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);

    /// Shortcut for `Color::rgb(0.0, 0.0, 1.0)`.
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
}

#[cfg(test)]
mod tests {
    use super::Color;

    #[test]
    fn rgb8_creation() {
        assert!(same_color(
            Color::rgba(0.2, 0.4, 0.6, 1.0),
            Color::rgb8(51, 102, 153)
        ));
    }

    #[test]
    fn hex_creation() {
        let expected = Color::rgba(0.2, 0.4, 0.6, 1.0);

        assert!(same_color(expected, Color::hex("336699")));
        assert!(same_color(expected, Color::hex("#336699")));
        assert!(same_color(expected, Color::hex("336699FF")));
        assert!(same_color(expected, Color::hex("#336699FF")));
    }

    #[test]
    fn try_hex_creation() {
        let expected = Color::rgba(0.2, 0.4, 0.6, 1.0);

        assert!(same_color(expected, Color::try_hex("336699").unwrap()));
        assert!(same_color(expected, Color::try_hex("#336699").unwrap()));
        assert!(same_color(expected, Color::try_hex("336699FF").unwrap()));
        assert!(same_color(expected, Color::try_hex("#336699FF").unwrap()));

        assert!(Color::try_hex("ZZZZZZ").is_err());
    }

    fn same_color(a: Color, b: Color) -> bool {
        (a.r - b.r).abs() < std::f32::EPSILON
            && (a.g - b.g).abs() < std::f32::EPSILON
            && (a.b - b.b).abs() < std::f32::EPSILON
            && (a.a - b.a).abs() < std::f32::EPSILON
    }
}
