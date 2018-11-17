#[derive(Copy, Clone)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn rgb(r: f32, g: f32, b: f32) -> Color {
        Color { r, g, b, a: 1.0 }
    }

    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

// We can't call our constructors in consts, so time for macros...
macro_rules! const_color {
    ($name:ident, $r:expr, $g:expr, $b:expr) => {
        pub const $name: Color = Color {
            r: $r,
            g: $g,
            b: $b,
            a: 1.0,
        };
    };
}

const_color!(BLACK, 0.0, 0.0, 0.0);
const_color!(WHITE, 1.0, 1.0, 1.0);
const_color!(RED, 1.0, 0.0, 0.0);
const_color!(GREEN, 0.0, 1.0, 0.0);
const_color!(BLUE, 0.0, 0.0, 1.0);
