pub type ColorComponent = f32;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Default for Color {
    fn default() -> Self {
        Self::TRANSPARENT
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

    pub const TRANSPARENT: Color = Color::rgba(0., 0., 0., 0.);
    pub const WHITE: Color = Color::rgba(1., 1., 1., 1.);
    pub const LIGHTGRAY: Color = Color::rgba(0.78, 0.78, 0.78, 1.);
    pub const GRAY: Color = Color::rgba(0.5, 0.5, 0.5, 1.);
    pub const DARKGRAY: Color = Color::rgba(0.31, 0.31, 0.31, 1.);
    pub const YELLOW: Color = Color::rgba(1., 1., 0., 1.);
    pub const GOLD: Color = Color::rgba(1., 0.78, 0., 1.);
    pub const ORANGE: Color = Color::rgba(1., 0.63, 0., 1.);
    pub const PINK: Color = Color::rgba(1., 0.75, 0.75, 1.);
    pub const RED: Color = Color::rgba(1., 0., 0., 1.);
    pub const GREEN: Color = Color::rgba(0., 1., 0., 1.);
    pub const BLUE: Color = Color::rgba(0., 0., 1., 1.);
    pub const PURPLE: Color = Color::rgba(0.5, 0., 0.5, 1.);
    pub const VIOLET: Color = Color::rgba(0.90, 0.5, 0.94, 1.);
    pub const BLACK: Color = Color::rgba(0., 0., 0., 1.);
    pub const MAGENTA: Color = Color::rgba(1., 0., 1., 1.);
}

impl From<[f32; 4]> for Color {
    fn from(value: [f32; 4]) -> Color {
        Color::rgba(value[0], value[1], value[2], value[3])
    }
}

impl From<(f32, f32, f32, f32)> for Color {
    fn from(value: (f32, f32, f32, f32)) -> Color {
        Color::rgba(value.0, value.1, value.2, value.3)
    }
}

impl From<Color> for [f32; 4] {
    fn from(value: Color) -> Self {
        [value.r, value.g, value.b, value.a]
    }
}

pub fn rgba(r: ColorComponent, g: ColorComponent, b: ColorComponent, a: ColorComponent) -> Color {
    Color::rgba(r, g, b, a)
}
