use std::fmt::{self, Display, Formatter};

/// Result of a parse.
pub type XmlResult<T> = Result<T, XmlError>;

/// Everything that can go wrong between a string and a [`Document`](crate::Document).
///
/// There are no positions here on purpose: every variant names the construct it
/// choked on, and carrying a byte offset through every variant costs more than
/// it is worth for a parser this size.
#[derive(Debug)]
pub enum XmlError {
    /// Something the grammar requires is not what came next.
    Unexpected {
        /// What the parser was looking for.
        expected: String,
        /// The character it found instead, or `None` at the end of the input.
        found: Option<char>,
    },
    /// The input ends in the middle of a construct, with nothing left to read.
    UnexpectedEnd,
    /// A construct started and its terminator never came: a comment without
    /// `-->`, a CDATA section without `]]>`, an unclosed tag.
    Unterminated(&'static str),
    /// A name is empty or begins with a character XML does not allow there.
    InvalidName(String),
    /// An attribute value is not wrapped in quotes.
    UnquotedAttribute(String),
    /// Two attributes on the same element share a name.
    DuplicateAttribute(String),
    /// A closing tag does not name the element it closes.
    MismatchedTag {
        /// The element the parser was inside.
        expected: String,
        /// The name in the closing tag.
        found: String,
    },
    /// `&name;` is not one of the five predefined entities.
    UnknownEntity(String),
    /// A character reference is malformed, or names a code point XML forbids.
    InvalidCharRef(String),
    /// The document holds more than one root element.
    MultipleRoots,
    /// The document holds no element at all.
    NoRoot,
    /// Elements are nested past the parser's depth limit.
    TooDeep(usize),
    /// The declaration names an encoding this crate cannot decode.
    UnsupportedEncoding(String),
    /// The bytes are not valid UTF-8.
    Utf8(std::str::Utf8Error),
    /// Reading from the caller's reader failed.
    Io(std::io::Error),
}

impl Display for XmlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unexpected { expected, found } => match found {
                Some(found) => write!(f, "expected {expected}, found '{found}'"),
                None => write!(f, "expected {expected}, found the end of the input"),
            },
            Self::UnexpectedEnd => write!(f, "the input ends inside a construct"),
            Self::Unterminated(what) => write!(f, "unterminated {what}"),
            Self::InvalidName(name) => write!(f, "invalid name '{name}'"),
            Self::UnquotedAttribute(name) => {
                write!(f, "the value of '{name}' is not quoted")
            }
            Self::DuplicateAttribute(name) => write!(f, "attribute '{name}' appears twice"),
            Self::MismatchedTag { expected, found } => {
                write!(f, "closing tag '{found}' does not close '{expected}'")
            }
            Self::UnknownEntity(entity) => write!(f, "unknown entity '{entity}'"),
            Self::InvalidCharRef(reference) => {
                write!(f, "invalid character reference '{reference}'")
            }
            Self::MultipleRoots => write!(f, "the document has more than one root element"),
            Self::NoRoot => write!(f, "the document has no root element"),
            Self::TooDeep(depth) => write!(f, "elements nested more than {depth} deep"),
            Self::UnsupportedEncoding(encoding) => {
                write!(f, "unsupported document encoding '{encoding}'")
            }
            Self::Utf8(error) => write!(f, "the input is not valid utf-8: {error}"),
            Self::Io(error) => write!(f, "could not read the document: {error}"),
        }
    }
}

impl std::error::Error for XmlError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Utf8(error) => Some(error),
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<std::io::Error> for XmlError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_mismatched_tag_names_both_elements() {
        let error = XmlError::MismatchedTag {
            expected: String::from("div"),
            found: String::from("span"),
        };
        assert_eq!(error.to_string(), "closing tag 'span' does not close 'div'");
    }

    #[test]
    fn read_errors_keep_their_source() {
        let io = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short read");
        let error = XmlError::from(io);
        assert!(std::error::Error::source(&error).is_some());
        assert_eq!(error.to_string(), "could not read the document: short read");
    }
}
