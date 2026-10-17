//! The paint values SVG writes: hex, `rgb()`, a keyword, or nothing at all.

use xs_layout::Color;

use crate::error::{SvgError, SvgResult};
use crate::values::number;

/// Reads a paint value.
///
/// `Ok(None)` is `none`, which is not the same as black: it is the shape being
/// asked not to paint. `currentColor` resolves to the document's own `color`
/// attribute, which is how a monochrome icon set gets its colour from CSS.
///
/// # Errors
///
/// [`SvgError::BadValue`] for anything that is not one of the forms above.
pub fn parse(value: &str, current_color: Option<Color>) -> SvgResult<Option<Color>> {
    let value = value.trim();
    match value {
        "none" => return Ok(None),
        "currentColor" => return Ok(Some(current_color.unwrap_or(Color::BLACK))),
        _ => {}
    }

    if let Some(hex) = value.strip_prefix('#') {
        return hex_color(hex, value).map(Some);
    }
    if let Some(arguments) = arguments_of(value, "rgb") {
        return rgb_function(arguments).map(Some);
    }
    if let Some(arguments) = arguments_of(value, "hsl") {
        return hsl_function(arguments).map(Some);
    }

    keyword(value).map(Some).ok_or_else(|| SvgError::BadValue {
        attribute: "paint",
        value: value.to_owned(),
    })
}

/// `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`.
fn hex_color(hex: &str, whole: &str) -> SvgResult<Color> {
    let bad = || SvgError::BadValue {
        attribute: "paint",
        value: whole.to_owned(),
    };
    let pair = |from: usize| -> SvgResult<u8> {
        let digits = hex.get(from..from + 2).ok_or_else(bad)?;
        u8::from_str_radix(digits, 16).map_err(|_| bad())
    };
    let short = |at: usize| -> SvgResult<u8> {
        hex.get(at..at + 1)
            .ok_or_else(bad)
            .and_then(|digit| u8::from_str_radix(digit, 16).map_err(|_| bad()))
    };
    // `#abc` is `#aabbcc`: each digit is doubled, which is what the spread is.
    let spread = |digit: u8| f32::from(digit * 17) / 255.0;

    match hex.len() {
        3 | 4 => Ok(Color::rgba(
            spread(short(0)?),
            spread(short(1)?),
            spread(short(2)?),
            if hex.len() == 4 {
                spread(short(3)?)
            } else {
                1.0
            },
        )),
        6 | 8 => Ok(Color::rgba(
            f32::from(pair(0)?) / 255.0,
            f32::from(pair(2)?) / 255.0,
            f32::from(pair(4)?) / 255.0,
            if hex.len() == 8 {
                f32::from(pair(6)?) / 255.0
            } else {
                1.0
            },
        )),
        _ => Err(bad()),
    }
}

/// `rgb()` and `rgba()`, with either a 0-255 number or a percentage per channel.
fn rgb_function(arguments: &str) -> SvgResult<Color> {
    let parts = components(arguments, 3)?;
    Ok(Color::rgba(
        channel(parts[0])?,
        channel(parts[1])?,
        channel(parts[2])?,
        match parts.get(3) {
            Some(alpha) => alpha_component(alpha)?,
            None => 1.0,
        },
    ))
}

/// `hsl()` and `hsla()`. [`Color`] holds no colour space to convert into, so the
/// conversion is done here.
fn hsl_function(arguments: &str) -> SvgResult<Color> {
    let parts = components(arguments, 3)?;
    let hue = number(parts[0], "paint")?.rem_euclid(360.0);
    let saturation = percent(parts[1])?;
    let lightness = percent(parts[2])?;

    // The chroma formulation: one pass, and the grey cases fall out of it rather
    // than needing a branch of their own.
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let secondary = chroma * (1.0 - ((hue / 60.0) % 2.0 - 1.0).abs());
    let (red, green, blue) = match hue {
        hue if hue < 60.0 => (chroma, secondary, 0.0),
        hue if hue < 120.0 => (secondary, chroma, 0.0),
        hue if hue < 180.0 => (0.0, chroma, secondary),
        hue if hue < 240.0 => (0.0, secondary, chroma),
        hue if hue < 300.0 => (secondary, 0.0, chroma),
        _ => (chroma, 0.0, secondary),
    };
    let match_lightness = lightness - chroma / 2.0;

    Ok(Color::rgba(
        red + match_lightness,
        green + match_lightness,
        blue + match_lightness,
        match parts.get(3) {
            Some(alpha) => alpha_component(alpha)?,
            None => 1.0,
        },
    ))
}

/// The inside of `name(...)` or `namea(...)`, if `value` is one.
fn arguments_of<'a>(value: &'a str, name: &str) -> Option<&'a str> {
    let rest = value.strip_prefix(name)?;
    let rest = rest.strip_prefix('a').unwrap_or(rest);
    let rest = rest.strip_prefix('(')?;
    rest.strip_suffix(')')
}

/// The arguments of a colour function, in either the comma separated form or the
/// space separated one with a slash before the alpha.
fn components<'a>(arguments: &'a str, want: usize) -> SvgResult<Vec<&'a str>> {
    let parts: Vec<&'a str> = arguments
        .split([' ', ',', '\t', '\n', '\r', '/'])
        .filter(|part| !part.is_empty())
        .collect();
    if parts.len() != want && parts.len() != want + 1 {
        return Err(SvgError::BadValue {
            attribute: "paint",
            value: arguments.to_owned(),
        });
    }
    Ok(parts)
}

/// An HSL saturation or lightness, which is a percentage however it is written.
fn percent(value: &str) -> SvgResult<f32> {
    Ok(number(value.trim_end().trim_end_matches('%'), "paint")?.clamp(0.0, 100.0) / 100.0)
}

/// One `rgb()` channel: a 0-255 number or a percentage.
fn channel(value: &str) -> SvgResult<f32> {
    let as_fraction = is_percentage(value);
    let value = number(value.trim_end().trim_end_matches('%'), "paint")?;
    let value = if as_fraction {
        value / 100.0
    } else {
        value / 255.0
    };
    Ok(value.clamp(0.0, 1.0))
}

/// An alpha, as a 0-1 number or a percentage.
fn alpha_component(value: &str) -> SvgResult<f32> {
    let as_fraction = is_percentage(value);
    let value = number(value.trim_end().trim_end_matches('%'), "paint")?;
    let value = if as_fraction { value / 100.0 } else { value };
    Ok(value.clamp(0.0, 1.0))
}

fn is_percentage(value: &str) -> bool {
    value.trim_end().ends_with('%')
}

/// The SVG 1.1 colour keywords, as the 24 bit values they were defined with.
fn keyword(value: &str) -> Option<Color> {
    if value.eq_ignore_ascii_case("transparent") {
        return Some(Color::rgba(0.0, 0.0, 0.0, 0.0));
    }
    let lower = value.to_ascii_lowercase();
    let (_, red, green, blue) = KEYWORDS.iter().find(|(name, ..)| *name == lower)?;
    Some(Color::rgba(
        f32::from(*red) / 255.0,
        f32::from(*green) / 255.0,
        f32::from(*blue) / 255.0,
        1.0,
    ))
}

/// The 147 keywords SVG 1.1 defines, from the same table as CSS Color 3.
/// Each is the 8 bit colour it was defined with, rather than a fraction, so the
/// table stays readable.
const KEYWORDS: &[(&str, u8, u8, u8)] = &[
    ("aliceblue", 0xF0, 0xF8, 0xFF),
    ("antiquewhite", 0xFA, 0xEB, 0xD7),
    ("aqua", 0x00, 0xFF, 0xFF),
    ("aquamarine", 0x7F, 0xFF, 0xD4),
    ("azure", 0xF0, 0xFF, 0xFF),
    ("beige", 0xF5, 0xF5, 0xDC),
    ("bisque", 0xFF, 0xE4, 0xC4),
    ("black", 0x00, 0x00, 0x00),
    ("blanchedalmond", 0xFF, 0xEB, 0xCD),
    ("blue", 0x00, 0x00, 0xFF),
    ("blueviolet", 0x8A, 0x2B, 0xE2),
    ("brown", 0xA5, 0x2A, 0x2A),
    ("burlywood", 0xDE, 0xB8, 0x87),
    ("cadetblue", 0x5F, 0x9E, 0xA0),
    ("chartreuse", 0x7F, 0xFF, 0x00),
    ("chocolate", 0xD2, 0x69, 0x1E),
    ("coral", 0xFF, 0x7F, 0x50),
    ("cornflowerblue", 0x64, 0x95, 0xED),
    ("cornsilk", 0xFF, 0xF8, 0xDC),
    ("crimson", 0xDC, 0x14, 0x3C),
    ("cyan", 0x00, 0xFF, 0xFF),
    ("darkblue", 0x00, 0x00, 0x8B),
    ("darkcyan", 0x00, 0x8B, 0x8B),
    ("darkgoldenrod", 0xB8, 0x86, 0x0B),
    ("darkgray", 0xA9, 0xA9, 0xA9),
    ("darkgreen", 0x00, 0x64, 0x00),
    ("darkgrey", 0xA9, 0xA9, 0xA9),
    ("darkkhaki", 0xBD, 0xB7, 0x6B),
    ("darkmagenta", 0x8B, 0x00, 0x8B),
    ("darkolivegreen", 0x55, 0x6B, 0x2F),
    ("darkorange", 0xFF, 0x8C, 0x00),
    ("darkorchid", 0x99, 0x32, 0xCC),
    ("darkred", 0x8B, 0x00, 0x00),
    ("darksalmon", 0xE9, 0x96, 0x7A),
    ("darkseagreen", 0x8F, 0xBC, 0x8F),
    ("darkslateblue", 0x48, 0x3D, 0x8B),
    ("darkslategray", 0x2F, 0x4F, 0x4F),
    ("darkslategrey", 0x2F, 0x4F, 0x4F),
    ("darkturquoise", 0x00, 0xCE, 0xD1),
    ("darkviolet", 0x94, 0x00, 0xD3),
    ("deeppink", 0xFF, 0x14, 0x93),
    ("deepskyblue", 0x00, 0xBF, 0xFF),
    ("dimgray", 0x69, 0x69, 0x69),
    ("dimgrey", 0x69, 0x69, 0x69),
    ("dodgerblue", 0x1E, 0x90, 0xFF),
    ("firebrick", 0xB2, 0x22, 0x22),
    ("floralwhite", 0xFF, 0xFA, 0xF0),
    ("forestgreen", 0x22, 0x8B, 0x22),
    ("fuchsia", 0xFF, 0x00, 0xFF),
    ("gainsboro", 0xDC, 0xDC, 0xDC),
    ("ghostwhite", 0xF8, 0xF8, 0xFF),
    ("gold", 0xFF, 0xD7, 0x00),
    ("goldenrod", 0xDA, 0xA5, 0x20),
    ("gray", 0x80, 0x80, 0x80),
    ("green", 0x00, 0x80, 0x00),
    ("greenyellow", 0xAD, 0xFF, 0x2F),
    ("grey", 0x80, 0x80, 0x80),
    ("honeydew", 0xF0, 0xFF, 0xF0),
    ("hotpink", 0xFF, 0x69, 0xB4),
    ("indianred", 0xCD, 0x5C, 0x5C),
    ("indigo", 0x4B, 0x00, 0x82),
    ("ivory", 0xFF, 0xFF, 0xF0),
    ("khaki", 0xF0, 0xE6, 0x8C),
    ("lavender", 0xE6, 0xE6, 0xFA),
    ("lavenderblush", 0xFF, 0xF0, 0xF5),
    ("lawngreen", 0x7C, 0xFC, 0x00),
    ("lemonchiffon", 0xFF, 0xFA, 0xCD),
    ("lightblue", 0xAD, 0xD8, 0xE6),
    ("lightcoral", 0xF0, 0x80, 0x80),
    ("lightcyan", 0xE0, 0xFF, 0xFF),
    ("lightgoldenrodyellow", 0xFA, 0xFA, 0xD2),
    ("lightgray", 0xD3, 0xD3, 0xD3),
    ("lightgreen", 0x90, 0xEE, 0x90),
    ("lightgrey", 0xD3, 0xD3, 0xD3),
    ("lightpink", 0xFF, 0xB6, 0xC1),
    ("lightsalmon", 0xFF, 0xA0, 0x7A),
    ("lightseagreen", 0x20, 0xB2, 0xAA),
    ("lightskyblue", 0x87, 0xCE, 0xFA),
    ("lightslategray", 0x77, 0x88, 0x99),
    ("lightslategrey", 0x77, 0x88, 0x99),
    ("lightsteelblue", 0xB0, 0xC4, 0xDE),
    ("lightyellow", 0xFF, 0xFF, 0xE0),
    ("lime", 0x00, 0xFF, 0x00),
    ("limegreen", 0x32, 0xCD, 0x32),
    ("linen", 0xFA, 0xF0, 0xE6),
    ("magenta", 0xFF, 0x00, 0xFF),
    ("maroon", 0x80, 0x00, 0x00),
    ("mediumaquamarine", 0x66, 0xCD, 0xAA),
    ("mediumblue", 0x00, 0x00, 0xCD),
    ("mediumorchid", 0xBA, 0x55, 0xD3),
    ("mediumpurple", 0x93, 0x70, 0xDB),
    ("mediumseagreen", 0x3C, 0xB3, 0x71),
    ("mediumslateblue", 0x7B, 0x68, 0xEE),
    ("mediumspringgreen", 0x00, 0xFA, 0x9A),
    ("mediumturquoise", 0x48, 0xD1, 0xCC),
    ("mediumvioletred", 0xC7, 0x15, 0x85),
    ("midnightblue", 0x19, 0x19, 0x70),
    ("mintcream", 0xF5, 0xFF, 0xFA),
    ("mistyrose", 0xFF, 0xE4, 0xE1),
    ("moccasin", 0xFF, 0xE4, 0xB5),
    ("navajowhite", 0xFF, 0xDE, 0xAD),
    ("navy", 0x00, 0x00, 0x80),
    ("oldlace", 0xFD, 0xF5, 0xE6),
    ("olive", 0x80, 0x80, 0x00),
    ("olivedrab", 0x6B, 0x8E, 0x23),
    ("orange", 0xFF, 0xA5, 0x00),
    ("orangered", 0xFF, 0x45, 0x00),
    ("orchid", 0xDA, 0x70, 0xD6),
    ("palegoldenrod", 0xEE, 0xE8, 0xAA),
    ("palegreen", 0x98, 0xFB, 0x98),
    ("paleturquoise", 0xAF, 0xEE, 0xEE),
    ("palevioletred", 0xDB, 0x70, 0x93),
    ("papayawhip", 0xFF, 0xEF, 0xD5),
    ("peachpuff", 0xFF, 0xDA, 0xB9),
    ("peru", 0xCD, 0x85, 0x3F),
    ("pink", 0xFF, 0xC0, 0xCB),
    ("plum", 0xDD, 0xA0, 0xDD),
    ("powderblue", 0xB0, 0xE0, 0xE6),
    ("purple", 0x80, 0x00, 0x80),
    ("rebeccapurple", 0x66, 0x33, 0x99),
    ("red", 0xFF, 0x00, 0x00),
    ("rosybrown", 0xBC, 0x8F, 0x8F),
    ("royalblue", 0x41, 0x69, 0xE1),
    ("saddlebrown", 0x8B, 0x45, 0x13),
    ("salmon", 0xFA, 0x80, 0x72),
    ("sandybrown", 0xF4, 0xA4, 0x60),
    ("seagreen", 0x2E, 0x8B, 0x57),
    ("seashell", 0xFF, 0xF5, 0xEE),
    ("sienna", 0xA0, 0x52, 0x2D),
    ("silver", 0xC0, 0xC0, 0xC0),
    ("skyblue", 0x87, 0xCE, 0xEB),
    ("slateblue", 0x6A, 0x5A, 0xCD),
    ("slategray", 0x70, 0x80, 0x90),
    ("slategrey", 0x70, 0x80, 0x90),
    ("snow", 0xFF, 0xFA, 0xFA),
    ("springgreen", 0x00, 0xFF, 0x7F),
    ("steelblue", 0x46, 0x82, 0xB4),
    ("tan", 0xD2, 0xB4, 0x8C),
    ("teal", 0x00, 0x80, 0x80),
    ("thistle", 0xD8, 0xBF, 0xD8),
    ("tomato", 0xFF, 0x63, 0x47),
    ("turquoise", 0x40, 0xE0, 0xD0),
    ("violet", 0xEE, 0x82, 0xEE),
    ("wheat", 0xF5, 0xDE, 0xB3),
    ("white", 0xFF, 0xFF, 0xFF),
    ("whitesmoke", 0xF5, 0xF5, 0xF5),
    ("yellow", 0xFF, 0xFF, 0x00),
    ("yellowgreen", 0x9A, 0xCD, 0x32),
];

#[cfg(test)]
mod tests {
    use super::*;

    fn read(value: &str) -> SvgResult<Option<Color>> {
        parse(value, None)
    }

    #[test]
    fn none_is_not_black() {
        let expected = None;
        assert_eq!(read("none").unwrap(), expected);
        assert_eq!(read(" none ").unwrap(), expected);
    }

    #[test]
    fn reads_six_digit_hex() {
        let expected = Color::rgba(1.0, 0.0, 0.0, 1.0);
        assert_eq!(read("#ff0000").unwrap(), Some(expected));
        assert_eq!(read("#FF0000").unwrap(), Some(expected));
    }

    #[test]
    fn reads_three_digit_hex_by_doubling_each_digit() {
        let expected = Color::rgba(0.0, f32::from(170_u8) / 255.0, 1.0, 1.0);
        assert_eq!(read("#0af").unwrap(), Some(expected));
    }

    #[test]
    fn reads_the_alpha_of_an_eight_digit_hex() {
        let expected = Color::rgba(0.0, 0.0, 0.0, f32::from(128_u8) / 255.0);
        assert_eq!(read("#00000080").unwrap(), Some(expected));
    }

    #[test]
    fn a_hex_of_the_wrong_length_is_an_error() {
        let error = read("#12345").unwrap_err();
        assert_eq!(error.to_string(), "'#12345' is not a valid paint");
    }

    #[test]
    fn a_hex_with_a_non_hex_digit_is_an_error() {
        let error = read("#zzz").unwrap_err();
        assert_eq!(error.to_string(), "'#zzz' is not a valid paint");
    }

    #[test]
    fn reads_rgb_in_both_spellings() {
        let expected = Color::rgba(1.0, 1.0, 0.0, 1.0);
        assert_eq!(read("rgb(255,255,0)").unwrap(), Some(expected));
        assert_eq!(read("rgb(255 255 0)").unwrap(), Some(expected));
    }

    #[test]
    fn reads_rgba_with_a_fraction_or_a_percentage_alpha() {
        let expected = Color::rgba(0.0, 0.0, 0.0, 0.5);
        assert_eq!(read("rgba(0,0,0,0.5)").unwrap(), Some(expected));
        assert_eq!(read("rgb(0 0 0 / 50%)").unwrap(), Some(expected));
    }

    #[test]
    fn reads_a_percentage_channel() {
        let expected = Color::rgba(1.0, 0.0, 0.0, 1.0);
        assert_eq!(read("rgb(100%, 0%, 0%)").unwrap(), Some(expected));
    }

    #[test]
    fn reads_hsl() {
        let expected = Color::RED;
        assert_eq!(read("hsl(0, 100%, 50%)").unwrap(), Some(expected));
        let expected = Color::GREEN;
        assert_eq!(read("hsl(120,100%,50%)").unwrap(), Some(expected));
        let expected = Color::rgba(1.0, 1.0, 1.0, 1.0);
        assert_eq!(read("hsl(0, 0%, 100%)").unwrap(), Some(expected));
    }

    #[test]
    fn reads_the_keywords_including_both_greys() {
        // 0x80 is not half of 0xFF, and the keyword table is 8 bit values.
        let grey = f32::from(128_u8) / 255.0;
        let expected = Color::rgba(grey, grey, grey, 1.0);
        assert_eq!(read("gray").unwrap(), Some(expected));
        assert_eq!(read("grey").unwrap(), Some(expected));

        let expected = Color::rgba(
            f32::from(0x66_u8) / 255.0,
            f32::from(0x33_u8) / 255.0,
            f32::from(0x99_u8) / 255.0,
            1.0,
        );
        assert_eq!(read("RebeccaPurple").unwrap(), Some(expected));
    }

    #[test]
    fn transparent_is_see_through_rather_than_a_keyword() {
        let expected = Color::TRANSPARENT;
        assert_eq!(read("transparent").unwrap(), Some(expected));
    }

    #[test]
    fn current_color_takes_the_documents_own_colour() {
        let expected = Color::rgba(0.0, 1.0, 1.0, 1.0);
        assert_eq!(
            parse("currentColor", Some(Color::rgba(0.0, 1.0, 1.0, 1.0))).unwrap(),
            Some(expected)
        );
    }

    #[test]
    fn current_color_falls_back_to_black() {
        let expected = Color::BLACK;
        assert_eq!(read("currentColor").unwrap(), Some(expected));
    }

    #[test]
    fn something_that_is_not_a_colour_is_an_error() {
        let error = read("chartreuse-ish").unwrap_err();
        let expected = "'chartreuse-ish' is not a valid paint";
        assert_eq!(error.to_string(), expected);
    }
}
