struct Pixels(f32);

struct AbsoluteLength(f32);
impl From<Pixels> for AbsoluteLength {
    fn from(pixels: Pixels) -> Self {
        AbsoluteLength::Pixels(pixels)
    }
}

pub fn px(pixels) -> Pixels {
    Pixels(pixels)
}
