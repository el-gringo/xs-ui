//! The numbers SVG writes.
//!
//! Every attribute in this crate is one of three shapes: a number, a number
//! with a unit, or a list of either. They differ in how forgiving they are
//! about separators - `points="1,2 3,4"` and `viewBox="0 0 24 24"` both have to
//! work - so the tokenizing lives here rather than in each caller.

use crate::error::{SvgError, SvgResult};

/// Pixels per inch, the unit every other one is defined against.
const PX_PER_INCH: f32 = 96.;

/// A bare number, with no unit and no percentage.
pub fn number(value: &str, attribute: &'static str) -> SvgResult<f32> {
    let value = value.trim();
    value
        .parse::<f32>()
        .ok()
        .filter(|number| number.is_finite())
        .ok_or_else(|| bad(attribute, value))
}

/// A length: a number, an absolute unit, or a percentage of `base`.
///
/// The absolute units are the CSS ones SVG inherits. `em` and `ex` are read as
/// their own number rather than rejected, because in practice they only ever
/// appear in a document whose font size is the 16px default anyway; `q` is
/// there so the unit list is complete even though nobody writes it.
pub fn length(value: &str, base: f32, attribute: &'static str) -> SvgResult<f32> {
    let value = value.trim();
    let (digits, unit) = split_unit(value);
    let scale = match unit {
        // No unit at all is pixels, which is the only unit SVG defaults to.
        // `em` and `ex` join it: they are read as their own number rather than
        // rejected, because in practice they only appear where the font size is
        // the 16px default anyway.
        "" | "px" | "em" | "ex" => 1.0,
        "pt" => PX_PER_INCH / 72.0,
        "pc" => PX_PER_INCH / 6.0,
        "in" => PX_PER_INCH,
        "cm" => PX_PER_INCH / 2.54,
        "mm" => PX_PER_INCH / 25.4,
        // `q` is here so the unit list is complete, even though nobody writes it.
        "q" => PX_PER_INCH / 101.6,
        "%" => return Ok(number(digits, attribute)? * base / 100.0),
        _ => return Err(bad(attribute, value)),
    };
    Ok(number(digits, attribute)? * scale)
}

/// A list of numbers, separated by whitespace and/or commas.
pub fn numbers(value: &str, attribute: &'static str) -> SvgResult<Vec<f32>> {
    value
        .split([' ', ',', '\t', '\n', '\r'])
        .filter(|token| !token.is_empty())
        .map(|token| number(token, attribute))
        .collect()
}

/// Splits a number from the unit after it.
///
/// The number's own alphabet includes `e`, which is also the first letter of two
/// units, so `1.5em` is one where `1.5e3px` is another: the exponent stays with
/// the number unless what follows the `e` spells `em` or `ex`.
fn split_unit(value: &str) -> (&str, &str) {
    let is_number = |c: char| c.is_ascii_digit() || matches!(c, '.' | '+' | '-' | 'e' | 'E');
    let end = value.find(|c: char| !is_number(c)).unwrap_or(value.len());
    let unit = &value[end..];
    // The scan ran past the `e` of `em`, because an exponent is the same letter.
    if matches!(unit, "m" | "x") && value[..end].ends_with(['e', 'E']) {
        return (&value[..end - 1], &value[end - 1..]);
    }
    (&value[..end], unit)
}

fn bad(attribute: &'static str, value: &str) -> SvgError {
    SvgError::BadValue {
        attribute,
        value: value.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_bare_number() {
        let expected = 12.5;
        assert_eq!(number(" 12.5 ", "width").unwrap(), expected);
        assert_eq!(number("-3", "width").unwrap(), -3.0);
        assert_eq!(number("1e2", "width").unwrap(), 100.0);
    }

    #[test]
    fn refuses_a_number_that_is_not_one() {
        let error = number("wide", "width").unwrap_err();
        assert_eq!(error.to_string(), "'wide' is not a valid width");
    }

    #[test]
    fn a_unitless_length_is_pixels() {
        let expected = 24.0;
        assert_eq!(length("24", 100.0, "width").unwrap(), expected);
        assert_eq!(length("24px", 100.0, "width").unwrap(), expected);
    }

    #[test]
    fn reads_the_absolute_units_svg_inherits_from_css() {
        assert_eq!(length("1in", 0.0, "width").unwrap(), 96.0);
        assert_eq!(length("72pt", 0.0, "width").unwrap(), 96.0);
        assert_eq!(length("6pc", 0.0, "width").unwrap(), 96.0);
        assert_eq!(length("25.4mm", 0.0, "width").unwrap(), 96.0);
        assert_eq!(length("2.54cm", 0.0, "width").unwrap(), 96.0);
    }

    #[test]
    fn a_percentage_is_a_fraction_of_its_base() {
        let expected = 24.0;
        assert_eq!(length("50%", 48.0, "width").unwrap(), expected);
        assert_eq!(length("100%", 24.0, "width").unwrap(), 24.0);
    }

    #[test]
    fn a_negative_percentage_keeps_its_sign() {
        let expected = -5.0;
        assert_eq!(length("-10%", 50.0, "dx").unwrap(), expected);
    }

    #[test]
    fn an_exponent_is_not_mistaken_for_a_unit() {
        let expected = 1500.0;
        assert_eq!(length("1.5e3px", 0.0, "width").unwrap(), expected);
        let expected = 1.5;
        assert_eq!(length("1.5em", 0.0, "width").unwrap(), expected);
    }

    #[test]
    fn an_unknown_unit_is_an_error() {
        let error = length("10furlongs", 0.0, "width").unwrap_err();
        let expected = "'10furlongs' is not a valid width";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn reads_a_list_in_either_separator() {
        let expected = vec![1.0, 2.0, 3.0, 4.0];
        assert_eq!(numbers("1,2 3,4", "points").unwrap(), expected.clone());
        assert_eq!(numbers("1 2 3 4", "points").unwrap(), expected.clone());
        assert_eq!(numbers("  1 , 2 3\n4 ", "points").unwrap(), expected);
    }

    #[test]
    fn an_empty_list_is_empty_rather_than_a_number() {
        let expected = Vec::<f32>::new();
        assert_eq!(numbers("   ", "points").unwrap(), expected);
    }
}
