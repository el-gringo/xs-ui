use std::fmt::{self, Display, Formatter};

/// Result of parsing a font.
pub type Result<T> = std::result::Result<T, FontError>;

/// Why a font could not be read.
///
/// Deliberately coarse: the only distinction that changes what a caller can do
/// is "this is not a TrueType file" from "this file is damaged".
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontError {
    /// The file ends before the structure being read does.
    Truncated(&'static str),
    /// A required table is absent.
    MissingTable(&'static str),
    /// A table is present but internally inconsistent.
    Malformed(&'static str),
    /// A valid font this implementation does not handle.
    UnsupportedFormat(&'static str),
    /// The glyph index is past the end of the font.
    GlyphOutOfRange(u16),
}

impl Display for FontError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated(what) => write!(f, "font ends inside {what}"),
            Self::MissingTable(name) => write!(f, "font has no {name} table"),
            Self::Malformed(what) => write!(f, "malformed {what}"),
            Self::UnsupportedFormat(what) => write!(f, "unsupported font: {what}"),
            Self::GlyphOutOfRange(index) => write!(f, "glyph {index} is past the end of the font"),
        }
    }
}

impl std::error::Error for FontError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_variant_names_itself() {
        let cases = [
            (FontError::Truncated("loca"), "font ends inside loca"),
            (FontError::MissingTable("glyf"), "font has no glyf table"),
            (FontError::Malformed("cmap 4"), "malformed cmap 4"),
            (
                FontError::UnsupportedFormat("CFF-flavoured OpenType"),
                "unsupported font: CFF-flavoured OpenType",
            ),
            (
                FontError::GlyphOutOfRange(9),
                "glyph 9 is past the end of the font",
            ),
        ];
        for (error, expected) in cases {
            assert_eq!(error.to_string(), expected);
        }
    }

    #[test]
    fn it_is_a_std_error() {
        let error = FontError::Truncated("glyf");
        let dynamic: &dyn std::error::Error = &error;
        assert!(dynamic.source().is_none());
    }
}
