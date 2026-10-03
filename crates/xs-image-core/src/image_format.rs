/// An image container we can identify from its leading bytes.
///
/// The discriminants are stable: they end up in the registry built by
/// `xs-image`, and callers match on them.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ImageFormat {
    /// JFIF / EXIF, decoded by libjpeg.
    Jpeg,
    /// ISO 15948, decoded by libpng.
    Png,
    /// RIFF container, decoded by libwebp.
    WebP,
    /// ISO 18181, decoded by libjxl.
    JpegXl,
}

impl ImageFormat {
    /// Every format this workspace knows about, whether or not a decoder for it
    /// is compiled in.
    pub const ALL: [ImageFormat; 4] = [
        ImageFormat::Jpeg,
        ImageFormat::Png,
        ImageFormat::WebP,
        ImageFormat::JpegXl,
    ];

    /// The bytes every file of this format starts with.
    ///
    /// For WebP this is only `RIFF`, since the payload length sits between it
    /// and the `WEBP` tag: [`ImageFormat::sniff`] checks both halves.
    #[must_use]
    pub const fn signature(self) -> &'static [u8] {
        match self {
            Self::Jpeg => &[0xFF, 0xD8, 0xFF],
            Self::Png => &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A],
            Self::WebP => b"RIFF",
            Self::JpegXl => &[0xFF, 0x0A],
        }
    }

    /// Identifies `data` by its magic bytes.
    ///
    /// The two container formats need more than a prefix: WebP is any RIFF
    /// file, so the `WEBP` form type at offset 8 has to be checked too, and
    /// JPEG XL has both a bare codestream and a box container signature.
    #[must_use]
    pub fn sniff(data: &[u8]) -> Option<ImageFormat> {
        // The `JXL ` signature box, unsigned (`0x0C` size) or signed
        // (`0xFF 0x0A` as the size tag) - either way a 12 byte prefix.
        const JXL_CONTAINER: &[u8] = &[
            0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
        ];

        if data.starts_with(JXL_CONTAINER) || data.starts_with(&[0xFF, 0x0A]) {
            return Some(Self::JpegXl);
        }
        if data.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return Some(Self::Jpeg);
        }
        if data.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
            return Some(Self::Png);
        }
        // RIFF puts an unvalidated length at offset 4, so the form type is the
        // only reliable marker.
        if data.starts_with(b"RIFF") && data.get(8..12) == Some(&b"WEBP"[..]) {
            return Some(Self::WebP);
        }
        None
    }

    /// Lowercase name, used in error messages and as the registry key.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Jpeg => "jpeg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::JpegXl => "jpeg-xl",
        }
    }

    /// The IANA media type.
    #[must_use]
    pub const fn mime_type(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Png => "image/png",
            Self::WebP => "image/webp",
            Self::JpegXl => "image/jxl",
        }
    }

    /// The file extension, without the dot.
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Png => "png",
            Self::WebP => "webp",
            Self::JpegXl => "jxl",
        }
    }
}

impl std::fmt::Display for ImageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sniffs_every_format_from_its_own_magic_bytes() {
        let expected = ImageFormat::Jpeg;
        assert_eq!(
            ImageFormat::sniff(&[0xFF, 0xD8, 0xFF, 0xE0]),
            Some(expected)
        );

        let expected = ImageFormat::Png;
        let png = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00];
        assert_eq!(ImageFormat::sniff(&png), Some(expected));

        let expected = ImageFormat::WebP;
        let webp = b"RIFF\x24\x00\x00\x00WEBPVP8 ";
        assert_eq!(ImageFormat::sniff(webp), Some(expected));

        let expected = ImageFormat::JpegXl;
        assert_eq!(ImageFormat::sniff(&[0xFF, 0x0A]), Some(expected));
    }

    #[test]
    fn sniffs_the_jpeg_xl_container_signature() {
        let container = [
            0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
        ];
        let expected = ImageFormat::JpegXl;
        assert_eq!(ImageFormat::sniff(&container), Some(expected));
    }

    #[test]
    fn a_riff_file_that_is_not_webp_is_not_webp() {
        let wav = b"RIFF\x24\x00\x00\x00WAVEfmt ";
        let expected = None;
        assert_eq!(ImageFormat::sniff(wav), expected);
    }

    #[test]
    fn an_empty_slice_sniffs_to_nothing() {
        let expected = None;
        assert_eq!(ImageFormat::sniff(&[]), expected);
    }

    #[test]
    fn a_truncated_signature_sniffs_to_nothing() {
        let expected = None;
        assert_eq!(ImageFormat::sniff(&[0x89, 0x50, 0x4E]), expected);
    }
}
