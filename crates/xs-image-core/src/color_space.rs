/// How the samples of an [`ImageBuffer`](crate::ImageBuffer) are laid out.
///
/// Every decoder normalises to one of these four, which is all
/// `glTexImage2D` can be handed without a swizzle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ColorSpace {
    /// One sample per pixel: grayscale.
    Luma,
    /// Grayscale plus opacity.
    LumaAlpha,
    /// Red, green, blue. Opaque.
    Rgb,
    /// Red, green, blue, opacity.
    Rgba,
}

impl ColorSpace {
    /// Every colour space, in increasing channel count.
    pub const ALL: [ColorSpace; 4] = [
        ColorSpace::Luma,
        ColorSpace::LumaAlpha,
        ColorSpace::Rgb,
        ColorSpace::Rgba,
    ];

    /// Number of samples stored per pixel.
    #[must_use]
    pub const fn channels(self) -> usize {
        match self {
            ColorSpace::Luma => 1,
            ColorSpace::LumaAlpha => 2,
            ColorSpace::Rgb => 3,
            ColorSpace::Rgba => 4,
        }
    }

    /// The colour space with this many channels, or `None` when the count is
    /// not one a pixel can have.
    #[must_use]
    pub const fn from_channels(channels: usize) -> Option<ColorSpace> {
        match channels {
            1 => Some(ColorSpace::Luma),
            2 => Some(ColorSpace::LumaAlpha),
            3 => Some(ColorSpace::Rgb),
            4 => Some(ColorSpace::Rgba),
            _ => None,
        }
    }

    /// Whether pixels carry an opacity sample.
    #[must_use]
    pub const fn has_alpha(self) -> bool {
        matches!(self, ColorSpace::LumaAlpha | ColorSpace::Rgba)
    }

    /// Lowercase name, matching the `name` of the matching
    /// [`ImageFormat`](crate::ImageFormat).
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            ColorSpace::Luma => "luma",
            ColorSpace::LumaAlpha => "luma-alpha",
            ColorSpace::Rgb => "rgb",
            ColorSpace::Rgba => "rgba",
        }
    }
}

impl std::fmt::Display for ColorSpace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn channel_count_round_trips_through_from_channels() {
        for color_space in ColorSpace::ALL {
            assert_eq!(
                ColorSpace::from_channels(color_space.channels()),
                Some(color_space)
            );
        }
    }

    #[test]
    fn from_channels_rejects_impossible_channel_counts() {
        assert_eq!(ColorSpace::from_channels(0), None);
        assert_eq!(ColorSpace::from_channels(5), None);
    }

    #[test]
    fn display_is_the_name() {
        let actual = ColorSpace::LumaAlpha.to_string();
        let expected = "luma-alpha";
        assert_eq!(actual, expected);
    }

    #[test]
    fn only_luma_alpha_and_rgba_carry_alpha() {
        assert!(!ColorSpace::Luma.has_alpha());
        assert!(!ColorSpace::Rgb.has_alpha());
        assert!(ColorSpace::LumaAlpha.has_alpha());
        assert!(ColorSpace::Rgba.has_alpha());
    }
}
