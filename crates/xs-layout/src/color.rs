// A color stored as RGBA components, each ranging from 0 - 255.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Color {
        colors::TRANSPARENT
    }
}

impl Color {
    #[must_use]
    pub const fn rgba(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color { r, g, b, a }
    }
}

impl From<[f32; 4]> for Color {
    fn from(lhs: [f32; 4]) -> Color {
        Color::rgba(lhs[0], lhs[1], lhs[2], lhs[3])
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(lhs: (f32, f32, f32, f32)) -> Color {
        let (r, g, b, a) = lhs;
        Color::rgba(r, g, b, a)
    }
}

// --------------------------------------------------
// Transform from Vector4
// --------------------------------------------------

impl From<Color> for [f32; 4] {
    fn from(lhs: Color) -> [f32; 4] {
        [lhs.r, lhs.g, lhs.b, lhs.a]
    }
}

impl From<Color> for (f32, f32, f32, f32) {
    fn from(lhs: Color) -> (f32, f32, f32, f32) {
        (lhs.r, lhs.g, lhs.b, lhs.a)
    }
}


pub mod colors {
    //! Constants for some common colors.
    use super::Color;

    pub const TRANSPARENT: Color    = Color::rgba(0.00, 0.00, 0.00, 0.00);
    pub const LIGHTGRAY: Color      = Color::rgba(0.78, 0.78, 0.78, 1.00);
    pub const GRAY: Color           = Color::rgba(0.51, 0.51, 0.51, 1.00);
    pub const DARKGRAY: Color       = Color::rgba(0.31, 0.31, 0.31, 1.00);
    pub const YELLOW: Color         = Color::rgba(0.99, 0.98, 0.00, 1.00);
    pub const GOLD: Color           = Color::rgba(1.00, 0.80, 0.00, 1.00);
    pub const ORANGE: Color         = Color::rgba(1.00, 0.63, 0.00, 1.00);
    pub const PINK: Color           = Color::rgba(1.00, 0.43, 0.76, 1.00);
    pub const RED: Color            = Color::rgba(0.90, 0.16, 0.22, 1.00);
    pub const MAROON: Color         = Color::rgba(0.75, 0.13, 0.22, 1.00);
    pub const GREEN: Color          = Color::rgba(0.00, 0.89, 0.19, 1.00);
    pub const LIME: Color           = Color::rgba(0.00, 0.62, 0.18, 1.00);
    pub const DARKGREEN: Color      = Color::rgba(0.00, 0.46, 0.17, 1.00);
    pub const SKYBLUE: Color        = Color::rgba(0.40, 0.75, 1.00, 1.00);
    pub const BLUE: Color           = Color::rgba(0.00, 0.47, 0.95, 1.00);
    pub const DARKBLUE: Color       = Color::rgba(0.00, 0.32, 0.67, 1.00);
    pub const PURPLE: Color         = Color::rgba(0.78, 0.48, 1.00, 1.00);
    pub const VIOLET: Color         = Color::rgba(0.53, 0.24, 0.75, 1.00);
    pub const DARKPURPLE: Color     = Color::rgba(0.44, 0.12, 0.49, 1.00);
    pub const BEIGE: Color          = Color::rgba(0.83, 0.69, 0.51, 1.00);
    pub const BROWN: Color          = Color::rgba(0.50, 0.42, 0.31, 1.00);
    pub const DARKBROWN: Color      = Color::rgba(0.30, 0.25, 0.18, 1.00);
    pub const WHITE: Color          = Color::rgba(1.00, 1.00, 1.00, 1.00);
    pub const BLACK: Color          = Color::rgba(0.00, 0.00, 0.00, 1.00);
    pub const BLANK: Color          = Color::rgba(0.00, 0.00, 0.00, 0.00);
    pub const MAGENTA: Color        = Color::rgba(1.00, 0.00, 1.00, 1.00);
}
