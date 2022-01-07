//! Functions and types relating to color.

use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

use crate::error::{Result, TetraError};
use crate::math::Vec4;

/// An RGBA color.
///
/// The components are stored as [`f32`] values, which will generally be
/// in the range of `0.0` to `1.0`.
///
/// If your data is made up of bytes or hex values, this type provides
/// constructors that will carry out the conversion for you.
///
/// The [`std` arithmetic traits](std::ops) are implemented for this type, which allows you to
/// add/subtract/multiply/divide colors.
///
/// # Serde
///
/// Serialization and deserialization of this type (via [Serde](https://serde.rs/))
/// can be enabled via the `serde_support` feature.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
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
    /// * [`TetraError::InvalidColor`] will be returned if the specified color is invalid.
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

    /// Returns the color with the red component set to the specified value.
    pub const fn with_red(self, r: f32) -> Self {
        Self { r, ..self }
    }

    /// Returns the color with the green component set to the specified value.
    pub const fn with_green(self, g: f32) -> Self {
        Self { g, ..self }
    }

    /// Returns the color with the blue component set to the specified value.
    pub const fn with_blue(self, b: f32) -> Self {
        Self { b, ..self }
    }

    /// Returns the color with the alpha component set to the specified value.
    pub const fn with_alpha(self, a: f32) -> Self {
        Self { a, ..self }
    }

    /// Returns the color with all components clamped between 0.0 and 1.0.
    pub fn clamp(self) -> Self {
        Color {
            r: clamp_f32(self.r),
            g: clamp_f32(self.g),
            b: clamp_f32(self.b),
            a: clamp_f32(self.a),
        }
    }

    /// Returns the color with the RGB components multiplied by the alpha component.
    ///
    /// This can be useful when working with
    /// [premultiplied alpha blending](super::BlendState::alpha), if
    /// you want to convert a non-premultiplied color into its premultiplied
    /// version.
    pub fn to_premultiplied(self) -> Color {
        Color {
            r: self.r * self.a,
            g: self.g * self.a,
            b: self.b * self.a,
            a: self.a,
        }
    }

    // These constants should remain at the bottom of the impl block to keep
    // the docs readable - don't want to have to scroll through a load of colors
    // to get to the methods!

    /// Shortcut for [`Color::rgb(0.0, 0.0, 0.0)`](Self::rgb).
    pub const BLACK: Color = Color::rgb(0.0, 0.0, 0.0);

    /// Shortcut for [`Color::rgb(1.0, 1.0, 1.0)`](Self::rgb).
    pub const WHITE: Color = Color::rgb(1.0, 1.0, 1.0);

    /// Shortcut for [`Color::rgb(1.0, 0.0, 0.0)`](Self::rgb).
    pub const RED: Color = Color::rgb(1.0, 0.0, 0.0);

    /// Shortcut for [`Color::rgb(0.0, 1.0, 0.0)`](Self::rgb).
    pub const GREEN: Color = Color::rgb(0.0, 1.0, 0.0);

    /// Shortcut for Color::rgb(0.0, 0.0, 1.0)`](Self::rgb).
    pub const BLUE: Color = Color::rgb(0.0, 0.0, 1.0);
}

fn clamp_f32(val: f32) -> f32 {
    f32::min(f32::max(0.0, val), 1.0)
}

impl From<Color> for Vec4<f32> {
    fn from(color: Color) -> Vec4<f32> {
        Vec4::new(color.r, color.g, color.b, color.a)
    }
}

impl From<Vec4<f32>> for Color {
    fn from(v: Vec4<f32>) -> Self {
        Color::rgba(v.x, v.y, v.z, v.w)
    }
}

impl From<Color> for [f32; 4] {
    fn from(color: Color) -> Self {
        [color.r, color.g, color.b, color.a]
    }
}

impl From<[f32; 4]> for Color {
    fn from(v: [f32; 4]) -> Self {
        Color::rgba(v[0], v[1], v[2], v[3])
    }
}

impl From<Color> for [u8; 4] {
    fn from(color: Color) -> Self {
        let clamped = color.clamp();

        [
            (clamped.r * 255.0) as u8,
            (clamped.g * 255.0) as u8,
            (clamped.b * 255.0) as u8,
            (clamped.a * 255.0) as u8,
        ]
    }
}

impl From<[u8; 4]> for Color {
    fn from(v: [u8; 4]) -> Self {
        Color::rgba8(v[0], v[1], v[2], v[3])
    }
}

impl Add for Color {
    type Output = Color;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;

        self
    }
}

impl Add<f32> for Color {
    type Output = Color;

    fn add(mut self, rhs: f32) -> Self::Output {
        self.r += rhs;
        self.g += rhs;
        self.b += rhs;
        self.a += rhs;

        self
    }
}

impl AddAssign for Color {
    fn add_assign(&mut self, rhs: Self) {
        self.r += rhs.r;
        self.g += rhs.g;
        self.b += rhs.b;
        self.a += rhs.a;
    }
}

impl AddAssign<f32> for Color {
    fn add_assign(&mut self, rhs: f32) {
        self.r += rhs;
        self.g += rhs;
        self.b += rhs;
        self.a += rhs;
    }
}

impl Sub for Color {
    type Output = Color;

    fn sub(mut self, rhs: Self) -> Self::Output {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self.a -= rhs.a;

        self
    }
}

impl Sub<f32> for Color {
    type Output = Color;

    fn sub(mut self, rhs: f32) -> Self::Output {
        self.r -= rhs;
        self.g -= rhs;
        self.b -= rhs;
        self.a -= rhs;

        self
    }
}

impl SubAssign for Color {
    fn sub_assign(&mut self, rhs: Self) {
        self.r -= rhs.r;
        self.g -= rhs.g;
        self.b -= rhs.b;
        self.a -= rhs.a;
    }
}

impl SubAssign<f32> for Color {
    fn sub_assign(&mut self, rhs: f32) {
        self.r -= rhs;
        self.g -= rhs;
        self.b -= rhs;
        self.a -= rhs;
    }
}

impl Mul for Color {
    type Output = Color;

    fn mul(mut self, rhs: Self) -> Self::Output {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
        self.a *= rhs.a;

        self
    }
}

impl Mul<f32> for Color {
    type Output = Color;

    fn mul(mut self, rhs: f32) -> Self::Output {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self.a *= rhs;

        self
    }
}

impl MulAssign for Color {
    fn mul_assign(&mut self, rhs: Self) {
        self.r *= rhs.r;
        self.g *= rhs.g;
        self.b *= rhs.b;
        self.a *= rhs.a;
    }
}

impl MulAssign<f32> for Color {
    fn mul_assign(&mut self, rhs: f32) {
        self.r *= rhs;
        self.g *= rhs;
        self.b *= rhs;
        self.a *= rhs;
    }
}

impl Div for Color {
    type Output = Color;

    fn div(mut self, rhs: Self) -> Self::Output {
        self.r /= rhs.r;
        self.g /= rhs.g;
        self.b /= rhs.b;
        self.a /= rhs.a;

        self
    }
}

impl Div<f32> for Color {
    type Output = Color;

    fn div(mut self, rhs: f32) -> Self::Output {
        self.r /= rhs;
        self.g /= rhs;
        self.b /= rhs;
        self.a /= rhs;

        self
    }
}

impl DivAssign for Color {
    fn div_assign(&mut self, rhs: Self) {
        self.r /= rhs.r;
        self.g /= rhs.g;
        self.b /= rhs.b;
        self.a /= rhs.a;
    }
}

impl DivAssign<f32> for Color {
    fn div_assign(&mut self, rhs: f32) {
        self.r = self.r / rhs;
        self.g = self.g / rhs;
        self.b = self.b / rhs;
        self.a = self.a / rhs;
    }
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

    #[test]
    fn to_premultiplied() {
        assert_eq!(
            Color::rgba(0.5, 0.5, 0.5, 0.5),
            Color::rgba(1.0, 1.0, 1.0, 0.5).to_premultiplied()
        );
    }

    #[test]
    fn ops() {
        assert_eq!(
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            Color::rgba(0.1, 0.2, 0.3, 0.4) + Color::rgba(0.9, 0.8, 0.7, 0.6)
        );

        assert_eq!(
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            Color::rgba(0.5, 0.5, 0.5, 0.5) + 0.5
        );

        assert_eq!(Color::rgba(1.0, 1.0, 1.0, 1.0), {
            let mut add_assign = Color::rgba(0.1, 0.2, 0.3, 0.4);
            add_assign += Color::rgba(0.9, 0.8, 0.7, 0.6);
            add_assign
        });

        assert_eq!(Color::rgba(1.0, 1.0, 1.0, 1.0), {
            let mut add_assign = Color::rgba(0.5, 0.5, 0.5, 0.5);
            add_assign += 0.5;
            add_assign
        });

        assert_eq!(
            Color::rgba(0.0, 0.0, 0.0, 0.0),
            Color::rgba(0.5, 0.5, 0.5, 0.5) - Color::rgba(0.5, 0.5, 0.5, 0.5)
        );

        assert_eq!(
            Color::rgba(0.0, 0.0, 0.0, 0.0),
            Color::rgba(0.5, 0.5, 0.5, 0.5) - 0.5
        );

        assert_eq!(Color::rgba(0.0, 0.0, 0.0, 0.0), {
            let mut sub_assign = Color::rgba(0.5, 0.5, 0.5, 0.5);
            sub_assign -= Color::rgba(0.5, 0.5, 0.5, 0.5);
            sub_assign
        });

        assert_eq!(Color::rgba(0.0, 0.0, 0.0, 0.0), {
            let mut sub_assign = Color::rgba(0.5, 0.5, 0.5, 0.5);
            sub_assign -= 0.5;
            sub_assign
        });

        assert_eq!(
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            Color::rgba(0.5, 0.5, 0.5, 0.5) * Color::rgba(2.0, 2.0, 2.0, 2.0)
        );

        assert_eq!(
            Color::rgba(1.0, 1.0, 1.0, 1.0),
            Color::rgba(0.5, 0.5, 0.5, 0.5) * 2.0
        );

        assert_eq!(Color::rgba(1.0, 1.0, 1.0, 1.0), {
            let mut mul_assign = Color::rgba(0.5, 0.5, 0.5, 0.5);
            mul_assign *= Color::rgba(2.0, 2.0, 2.0, 2.0);
            mul_assign
        });

        assert_eq!(Color::rgba(1.0, 1.0, 1.0, 1.0), {
            let mut mul_assign = Color::rgba(0.5, 0.5, 0.5, 0.5);
            mul_assign *= 2.0;
            mul_assign
        });

        assert_eq!(
            Color::rgba(0.5, 0.5, 0.5, 0.5),
            Color::rgba(1.0, 1.0, 1.0, 1.0) / Color::rgba(2.0, 2.0, 2.0, 2.0)
        );

        assert_eq!(
            Color::rgba(0.5, 0.5, 0.5, 0.5),
            Color::rgba(1.0, 1.0, 1.0, 1.0) / 2.0
        );

        assert_eq!(Color::rgba(0.5, 0.5, 0.5, 0.5), {
            let mut div_assign = Color::rgba(1.0, 1.0, 1.0, 1.0);
            div_assign /= Color::rgba(2.0, 2.0, 2.0, 2.0);
            div_assign
        });

        assert_eq!(Color::rgba(0.5, 0.5, 0.5, 0.5), {
            let mut div_assign = Color::rgba(1.0, 1.0, 1.0, 1.0);
            div_assign /= 2.0;
            div_assign
        });
    }

    fn same_color(a: Color, b: Color) -> bool {
        (a.r - b.r).abs() < std::f32::EPSILON
            && (a.g - b.g).abs() < std::f32::EPSILON
            && (a.b - b.b).abs() < std::f32::EPSILON
            && (a.a - b.a).abs() < std::f32::EPSILON
    }
}
