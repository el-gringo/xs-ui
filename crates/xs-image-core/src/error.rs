use std::fmt::{self, Display, Formatter};

use crate::{ColorSpace, ImageFormat};

/// Result of a decode attempt.
pub type ImageResult<T> = Result<T, ImageError>;

/// Everything that can go wrong between a byte slice and an
/// [`ImageBuffer`](crate::ImageBuffer).
#[derive(Debug)]
pub enum ImageError {
    /// No registered decoder claims the leading bytes.
    UnknownFormat,
    /// The bytes identify a format we know, but no decoder is registered for
    /// it: either the crate was compiled out or the shared library is missing.
    DecoderUnavailable(ImageFormat),
    /// The file header parsed, but the pixel data stops early.
    Truncated,
    /// The codec rejected the data, carrying the library's own message.
    InvalidData(String),
    /// The header declares a pixel count that overflows or cannot be allocated.
    DimensionsOverflow,
    /// The allocator refused the buffer.
    AllocationFailed,
    /// The image is zero pixels wide or tall.
    ZeroSized,
    /// The caller asked for a colour space this crate cannot produce.
    UnsupportedConversion(ColorSpace, ColorSpace),
    /// Reading from the caller supplied reader failed.
    Io(std::io::Error),
}

impl Display for ImageError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownFormat => write!(f, "unrecognised image format"),
            Self::DecoderUnavailable(format) => {
                write!(f, "no decoder available for {format}")
            }
            Self::Truncated => write!(f, "image data ends before the pixels do"),
            Self::InvalidData(reason) => write!(f, "invalid image data: {reason}"),
            Self::DimensionsOverflow => write!(f, "image dimensions overflow"),
            Self::AllocationFailed => write!(f, "could not allocate the pixel buffer"),
            Self::ZeroSized => write!(f, "image has no pixels"),
            Self::UnsupportedConversion(from, to) => {
                write!(f, "cannot convert {from} to {to}")
            }
            Self::Io(error) => write!(f, "could not read image: {error}"),
        }
    }
}

impl std::error::Error for ImageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<&str> for ImageError {
    fn from(reason: &str) -> Self {
        Self::InvalidData(reason.to_owned())
    }
}

impl From<String> for ImageError {
    fn from(reason: String) -> Self {
        Self::InvalidData(reason)
    }
}

impl From<std::io::Error> for ImageError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unknown_format_names_the_missing_bytes() {
        let expected = "unrecognised image format";
        assert_eq!(ImageError::UnknownFormat.to_string(), expected);
    }

    #[test]
    fn decoder_unavailable_names_the_format() {
        let error = ImageError::DecoderUnavailable(ImageFormat::JpegXl);
        assert_eq!(error.to_string(), "no decoder available for jpeg-xl");
    }

    #[test]
    fn an_unsupported_conversion_names_both_colour_spaces() {
        let error = ImageError::UnsupportedConversion(ColorSpace::Luma, ColorSpace::Rgba);
        assert_eq!(error.to_string(), "cannot convert luma to rgba");
    }

    #[test]
    fn io_errors_keep_their_source() {
        let io = std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "short read");
        let error = ImageError::from(io);
        assert!(std::error::Error::source(&error).is_some());
        assert_eq!(error.to_string(), "could not read image: short read");
    }
}
