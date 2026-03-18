pub type ColorComponent = u8;

// A color stored as RGBA components, each ranging from 0 - 255.
#[derive(PartialEq, Eq, Copy, Clone, Debug, Hash)]
pub struct Color {
    pub r: ColorComponent,
    pub g: ColorComponent,
    pub b: ColorComponent,
    pub a: ColorComponent,
}

impl Default for Color {
    fn default() -> Color {
        colors::TRANSPARENT
    }
}

impl Color {
    #[must_use]
    pub const fn rgba(
        r: ColorComponent,
        g: ColorComponent,
        b: ColorComponent,
        a: ColorComponent,
    ) -> Color {
        Color { r, g, b, a }
    }

    #[must_use]
    pub fn rgba_f32(r: f32, g: f32, b: f32, a: f32) -> Color {
        Color {
            r: (r * 255.) as u8,
            g: (g * 255.) as u8,
            b: (b * 255.) as u8,
            a: (a * 255.) as u8,
        }
    }
}

impl From<[f32; 4]> for Color {
    fn from(lhs: [f32; 4]) -> Color {
        Color::rgba(
            (lhs[0] * 255.) as _,
            (lhs[1] * 255.) as _,
            (lhs[2] * 255.) as _,
            (lhs[3] * 255.) as _,
        )
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(lhs: (f32, f32, f32, f32)) -> Color {
        Color::rgba(
            (lhs.0 * 255.) as _,
            (lhs.1 * 255.) as _,
            (lhs.2 * 255.) as _,
            (lhs.3 * 255.) as _,
        )
    }
}

// --------------------------------------------------
// Transform from Vector4
// --------------------------------------------------

impl From<Color> for [f32; 4] {
    fn from(lhs: Color) -> [f32; 4] {
        [
            lhs.r as f32 / 255.,
            lhs.g as f32 / 255.,
            lhs.b as f32 / 255.,
            lhs.a as f32 / 255.,
        ]
    }
}

impl From<&Color> for (f32, f32, f32, f32) {
    fn from(lhs: &Color) -> (f32, f32, f32, f32) {
        (
            lhs.r as f32 / 255.,
            lhs.g as f32 / 255.,
            lhs.b as f32 / 255.,
            lhs.a as f32 / 255.,
        )
    }
}

pub mod colors {
    //! Constants for some common colors.
    use super::Color;

    pub const TRANSPARENT: Color = Color::rgba(0, 0, 0, 0);
    pub const WHITE: Color = Color::rgba(255, 255, 255, 255);
    pub const LIGHTGRAY: Color = Color::rgba(200, 200, 200, 255);
    pub const GRAY: Color = Color::rgba(128, 128, 128, 255);
    pub const DARKGRAY: Color = Color::rgba(80, 80, 80, 255);
    pub const YELLOW: Color = Color::rgba(255, 255, 0, 255);
    pub const GOLD: Color = Color::rgba(255, 200, 0, 255);
    pub const ORANGE: Color = Color::rgba(255, 160, 0, 255);
    pub const PINK: Color = Color::rgba(255, 192, 203, 255);
    pub const RED: Color = Color::rgba(255, 0, 0, 255);
    pub const GREEN: Color = Color::rgba(0, 255, 0, 255);
    pub const BLUE: Color = Color::rgba(0, 0, 255, 255);
    pub const PURPLE: Color = Color::rgba(128, 0, 128, 255);
    pub const VIOLET: Color = Color::rgba(238, 130, 238, 255);
    pub const BLACK: Color = Color::rgba(0, 0, 0, 255);
    pub const MAGENTA: Color = Color::rgba(255, 0, 255, 255);
}
