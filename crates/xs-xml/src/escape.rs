use std::borrow::Cow;

use crate::error::{XmlError, XmlResult};

/// Resolves the character references in `input`.
///
/// The five predefined entities - `&lt;`, `&gt;`, `&amp;`, `&apos;` and
/// `&quot;` - are always available. Anything else has to come from the internal
/// DTD subset, which this crate does not parse, so an unknown entity is an
/// error rather than something passed through untouched.
///
/// # Errors
///
/// [`XmlError::UnknownEntity`] for a reference that is not one of the five,
/// [`XmlError::InvalidCharRef`] for one that is unterminated, not a number, or
/// names a code point XML forbids.
pub fn unescape(input: &str) -> XmlResult<Cow<'_, str>> {
    let Some(first) = input.find('&') else {
        return Ok(Cow::Borrowed(input));
    };

    let mut out = String::with_capacity(input.len());
    out.push_str(&input[..first]);
    let mut rest = &input[first..];

    while let Some(start) = rest.find('&') {
        out.push_str(&rest[..start]);
        let entity = &rest[start..];
        // An unterminated reference runs to the end of the text it is in, so
        // there is no ';' to find and nothing to resolve.
        let Some(end) = entity.find(';') else {
            return Err(XmlError::UnknownEntity(entity.to_owned()));
        };
        resolve(&mut out, &entity[..=end])?;
        rest = &entity[end + 1..];
    }
    out.push_str(rest);

    Ok(Cow::Owned(out))
}

/// Replaces the characters that cannot appear literally in text or in an
/// attribute value.
///
/// Escaping all five is more than any one context needs - text only needs three
/// of them - but it is the encoding that is always correct, so there is one
/// function rather than two.
#[must_use]
pub fn escape(input: &str) -> Cow<'_, str> {
    let Some(first) = input.find(['&', '<', '>', '"', '\'']) else {
        return Cow::Borrowed(input);
    };

    let mut out = String::with_capacity(input.len() + 16);
    out.push_str(&input[..first]);
    let mut rest = &input[first..];

    while let Some(start) = rest.find(['&', '<', '>', '"', '\'']) {
        out.push_str(&rest[..start]);
        match rest.as_bytes()[start] {
            b'&' => out.push_str("&amp;"),
            b'<' => out.push_str("&lt;"),
            b'>' => out.push_str("&gt;"),
            b'"' => out.push_str("&quot;"),
            _ => out.push_str("&apos;"),
        }
        rest = &rest[start + 1..];
    }
    out.push_str(rest);

    Cow::Owned(out)
}

/// Pushes one `&...;` reference onto `out`.
fn resolve(out: &mut String, entity: &str) -> XmlResult<()> {
    match entity {
        "&lt;" => out.push('<'),
        "&gt;" => out.push('>'),
        "&amp;" => out.push('&'),
        "&apos;" => out.push('\''),
        "&quot;" => out.push('"'),
        _ => {
            let body = entity
                .strip_prefix("&#")
                .ok_or_else(|| XmlError::UnknownEntity(entity.to_owned()))?;
            // `entity` is `&...;` whole, so the number is what is left once the
            // `&`, the `#` and the `;` are off it.
            let digits = body.strip_suffix(';').unwrap_or(body);
            let code = match digits.strip_prefix(['x', 'X']) {
                Some(hexadecimal) => u32::from_str_radix(hexadecimal, 16),
                None => digits.parse::<u32>(),
            }
            .map_err(|_| XmlError::InvalidCharRef(entity.to_owned()))?;
            let character = char::from_u32(code)
                .filter(|character| is_xml_char(u32::from(*character)))
                .ok_or_else(|| XmlError::InvalidCharRef(entity.to_owned()))?;
            out.push(character);
        }
    }
    Ok(())
}

/// The characters XML 1.0 allows in a document: the printable range minus the
/// C0 controls that are not whitespace, plus the two surrogate-free gaps at
/// either end of the supplementary planes.
fn is_xml_char(code: u32) -> bool {
    matches!(
        code,
        0x9 | 0xA | 0xD | 0x20..=0xD7FF | 0xE000..=0xFFFD | 0x0010_0000..=0x0010_FFFF
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_without_a_reference_is_borrowed() {
        let expected = "plain text";
        assert!(matches!(unescape(expected), Ok(Cow::Borrowed(_))));
        assert_eq!(unescape(expected).unwrap(), expected);
    }

    #[test]
    fn resolves_the_five_predefined_entities() {
        let input = "&lt;p&gt; a &amp; b &apos;c&apos; &quot;d&quot;";
        let expected = "<p> a & b 'c' \"d\"";
        assert_eq!(unescape(input).unwrap(), expected);
    }

    #[test]
    fn resolves_decimal_and_hexadecimal_references() {
        let expected = "A\u{00e9}\u{4e2d}";
        assert_eq!(unescape("&#65;&#233;&#x4e2d;").unwrap(), expected);
    }

    #[test]
    fn resolves_references_around_ordinary_text() {
        let expected = "one & two";
        assert_eq!(unescape("one &amp; two").unwrap(), expected);
        assert_eq!(unescape("&amp;&amp;").unwrap(), "&&");
        assert_eq!(unescape("&amp;amp;").unwrap(), "&amp;");
    }

    #[test]
    fn an_unterminated_reference_is_an_error() {
        let error = unescape("&amp").unwrap_err();
        assert_eq!(error.to_string(), "unknown entity '&amp'");
    }

    #[test]
    fn an_entity_from_an_unparsed_dtd_is_an_error() {
        let error = unescape("&nbsp;").unwrap_err();
        assert_eq!(error.to_string(), "unknown entity '&nbsp;'");
    }

    #[test]
    fn a_reference_outside_the_allowed_range_is_an_error() {
        let expected = "invalid character reference '&#0;'";
        assert_eq!(unescape("&#0;").unwrap_err().to_string(), expected);

        let expected = "invalid character reference '&#xD800;'";
        assert_eq!(unescape("&#xD800;").unwrap_err().to_string(), expected);
    }

    #[test]
    fn a_malformed_reference_is_an_error() {
        let expected = "invalid character reference '&#;'";
        assert_eq!(unescape("&#;").unwrap_err().to_string(), expected);

        let expected = "invalid character reference '&#xZZ;'";
        assert_eq!(unescape("&#xZZ;").unwrap_err().to_string(), expected);
    }

    #[test]
    fn escaping_covers_every_marked_character() {
        let expected = "&lt;a&gt; &amp; &apos; &quot;";
        assert_eq!(escape("<a> & ' \""), expected);
    }

    #[test]
    fn text_with_nothing_to_escape_is_borrowed() {
        let expected = "plain text";
        assert!(matches!(escape(expected), Cow::Borrowed(_)));
        assert_eq!(escape(expected), expected);
    }

    #[test]
    fn escaping_then_resolving_is_a_round_trip() {
        let original = "<a href=\"x\"> & ' </a>";
        let expected = original;
        assert_eq!(unescape(&escape(original)).unwrap(), expected);
    }
}
