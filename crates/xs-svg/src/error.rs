use std::fmt::{self, Display, Formatter};

/// Result of an SVG parse.
pub type SvgResult<T> = Result<T, SvgError>;

/// Everything that can go wrong between a document and a list of
/// [`Shape`](xs_vector::Shape)s.
///
/// The errors are the ones a caller can do something about. An element this
/// parser does not draw is not among them: `<linearGradient>` is skipped
/// silently rather than reported, because a real file is full of features that
/// have nothing to do with what ends up on screen.
#[derive(Debug)]
pub enum SvgError {
    /// The document is not well formed XML.
    Xml(xs_xml::XmlError),
    /// The document parsed, but its root element is not `<svg>`.
    NotSvg(String),
    /// An attribute is not the value its name promises.
    ///
    /// Every one of these names the attribute, because "invalid number" in a
    /// thousand line file is not a diagnosis: `stroke-width="1.5px"` says which
    /// one.
    BadValue {
        /// The attribute that would not read.
        attribute: &'static str,
        /// What was written instead.
        value: String,
    },
    /// A `d` attribute is not valid path data.
    Path(xs_vector::PathError),
    /// A `<use>` points at an id that is not in the document.
    ///
    /// An unresolved reference is an error rather than a skip: a `<use>` that
    /// draws nothing is an icon that is mysteriously half there.
    UnknownReference(String),
    /// `<use>` elements nested deeper than the walker follows.
    ///
    /// `<use href="#a"/>` where `a` contains a `<use href="#theRoot"/>` is a
    /// cycle, and following it is a stack overflow rather than a hang.
    RecursiveUse,
}

impl Display for SvgError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Xml(error) => write!(f, "the document is not valid xml: {error}"),
            Self::NotSvg(name) => write!(f, "the root element is <{name}>, not <svg>"),
            Self::BadValue { attribute, value } => {
                write!(f, "'{value}' is not a valid {attribute}")
            }
            Self::Path(error) => write!(f, "invalid path data: {error}"),
            Self::UnknownReference(id) => write!(f, "no element with id '{id}'"),
            Self::RecursiveUse => write!(f, "<use> elements are nested too deeply"),
        }
    }
}

impl std::error::Error for SvgError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Xml(error) => Some(error),
            Self::Path(error) => Some(error),
            _ => None,
        }
    }
}

impl From<xs_xml::XmlError> for SvgError {
    fn from(error: xs_xml::XmlError) -> Self {
        Self::Xml(error)
    }
}

impl From<xs_vector::PathError> for SvgError {
    fn from(error: xs_vector::PathError) -> Self {
        Self::Path(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bad_value_names_its_attribute() {
        let error = SvgError::BadValue {
            attribute: "stroke-width",
            value: String::from("thick"),
        };
        let expected = "'thick' is not a valid stroke-width";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn a_bad_root_element_names_itself() {
        let error = SvgError::NotSvg(String::from("html"));
        let expected = "the root element is <html>, not <svg>";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn xml_errors_keep_their_source() {
        let error = SvgError::from(xs_xml::XmlError::NoRoot);
        assert!(std::error::Error::source(&error).is_some());
    }
}
