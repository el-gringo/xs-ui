use xs_image_core::{
    ColorSpace, ImageBuffer, ImageDecoder, ImageDimensions, ImageError, ImageFormat, ImageResult,
};

use crate::libwebp_sys::{LibWebP, WebPBuffer};

/// Decodes WebP with `libwebp`.
///
/// Always produces [`ColorSpace::Rgba`]: libwebp's RGBA entry point is the one
/// that handles alpha without a second pass, and the renderer wants four
/// channels anyway. Alpha is straight, not premultiplied, which is what
/// `glTexImage2D` expects.
///
/// Lossy files come back at their stored dimensions; `RGBA` decoding does not
/// do the fancy upscaling `WebPGetFeatures` can report on.
///
/// # Panics
///
/// [`WebPDecoder::new`] panics when `libwebp.so` cannot be loaded; use
/// [`WebPDecoder::try_new`] to handle that.
///
/// # Examples
///
/// ```
/// use xs_image_core::ImageDecoder;
/// use xs_image_webp::WebPDecoder;
///
/// let decoder = WebPDecoder::new();
/// let image = decoder.decode(include_bytes!("../testdata/rgba.webp"))?;
/// assert_eq!(image.color_space(), xs_image_core::ColorSpace::Rgba);
/// # Ok::<(), xs_image_core::ImageError>(())
/// ```
#[derive(Clone)]
pub struct WebPDecoder {
    webp: LibWebP,
}

impl WebPDecoder {
    /// Loads `libwebp.so`.
    ///
    /// # Panics
    ///
    /// Panics when the shared library or any of its symbols is missing.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("failed to load libwebp.so")
    }

    /// Loads `libwebp.so`, reporting failure instead of panicking.
    ///
    /// # Errors
    ///
    /// [`ImageError::DecoderUnavailable`] when the library is not on this device.
    pub fn try_new() -> ImageResult<Self> {
        LibWebP::try_load()
            .map(|webp| Self { webp })
            .map_err(|_| ImageError::DecoderUnavailable(ImageFormat::WebP))
    }

    /// The linked libwebp version as `MAJOR * 10000 + MINOR * 100 + PATCH`.
    #[must_use]
    pub fn version(&self) -> i32 {
        unsafe { (self.webp.WebPGetDecoderVersion)() }
    }
}

impl Default for WebPDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDecoder for WebPDecoder {
    fn format(&self) -> ImageFormat {
        ImageFormat::WebP
    }

    fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions> {
        let (width, height) = self.info(data)?;
        Ok(ImageDimensions { width, height })
    }

    fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer> {
        if !Self::has_signature(data) {
            return Err(ImageError::UnknownFormat);
        }

        // libwebp both allocates and sizes the buffer, so the dimensions only
        // become known here. WebPBuffer gives the allocation back on every path
        // out of this function.
        let mut width = 0;
        let mut height = 0;
        let decoded = unsafe {
            (self.webp.WebPDecodeRGBA)(data.as_ptr(), data.len(), &raw mut width, &raw mut height)
        };
        let decoded = unsafe { WebPBuffer::new(decoded, self.webp.WebPFree) }.ok_or(
            ImageError::InvalidData("libwebp rejected the bitstream".into()),
        )?;

        let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
            return Err(ImageError::DimensionsOverflow);
        };
        let mut image = ImageBuffer::try_new(width, height, ColorSpace::Rgba)?;
        let len = image.len();
        image.as_mut_slice().copy_from_slice(decoded.as_slice(len));
        Ok(image)
    }
}

impl WebPDecoder {
    /// Whether `data` starts with `RIFF....WEBP`.
    fn has_signature(data: &[u8]) -> bool {
        data.starts_with(ImageFormat::WebP.signature()) && data.get(8..12) == Some(&b"WEBP"[..])
    }

    /// Reads the frame header into a `(width, height)` pair.
    fn info(&self, data: &[u8]) -> ImageResult<(u32, u32)> {
        if !Self::has_signature(data) {
            return Err(ImageError::UnknownFormat);
        }
        let mut width = 0;
        let mut height = 0;
        let valid = unsafe {
            (self.webp.WebPGetInfo)(data.as_ptr(), data.len(), &raw mut width, &raw mut height)
        };
        if valid == 0 {
            return Err(ImageError::InvalidData("malformed webp header".into()));
        }
        // libwebp writes these as int, but only ever non-negative sizes.
        let (Ok(width), Ok(height)) = (u32::try_from(width), u32::try_from(height)) else {
            return Err(ImageError::DimensionsOverflow);
        };
        if width == 0 || height == 0 {
            return Err(ImageError::ZeroSized);
        }
        Ok((width, height))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RGBA_WEBP: &[u8] = include_bytes!("../testdata/rgba.webp");

    /// The four colours baked into every fixture.
    ///
    /// The last pixel is white at alpha 1: near enough invisible to look like a
    /// premultiplication test, and a premultiplying decoder would report it as
    /// (1, 1, 1, 1) instead.
    const EXPECTED: [u8; 16] = [
        255, 0, 0, 255, // red, opaque
        0, 255, 0, 128, // green, half transparent
        0, 0, 255, 255, // blue, opaque
        255, 255, 255, 1, // white, almost invisible
    ];

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------
    fn decoder() -> WebPDecoder {
        WebPDecoder::try_new().expect("libwebp.so is not installed")
    }

    // --------------------------------------------------
    // Tests
    // --------------------------------------------------
    #[test]
    fn decodes_a_webp_to_rgba() {
        let image = decoder().decode(RGBA_WEBP).unwrap();
        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.color_space(), ColorSpace::Rgba);
        assert_eq!(image.len(), 2 * 2 * 4);
    }

    #[test]
    fn decodes_the_right_pixels() {
        // Lossless, so exact.
        let actual = decoder().decode(RGBA_WEBP).unwrap().into_vec();
        test_utils::assert_slice_eq(&EXPECTED, &actual);
    }

    #[test]
    fn alpha_comes_back_straight_and_not_premultiplied() {
        let image = decoder().decode(RGBA_WEBP).unwrap();
        let pixels = image.as_slice();
        let white = &pixels[12..16];
        let expected = [255, 255, 255, 1];
        test_utils::assert_slice_eq(&expected, white);
    }

    #[test]
    fn reports_dimensions_without_decoding() {
        let actual = decoder().read_dimensions(RGBA_WEBP).unwrap();
        let expected = ImageDimensions {
            width: 2,
            height: 2,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn the_decoded_size_matches_the_reported_one() {
        let decoder = decoder();
        let dimensions = decoder.read_dimensions(RGBA_WEBP).unwrap();
        let image = decoder.decode(RGBA_WEBP).unwrap();
        assert_eq!(image.dimensions(), dimensions.as_tuple());
    }

    #[test]
    fn reports_a_linked_version() {
        assert!(decoder().version() >= 0x10000, "unexpected version");
    }

    #[test]
    fn rejects_a_png() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let actual = decoder().decode(&png).unwrap_err();
        assert!(matches!(actual, ImageError::UnknownFormat));
    }

    #[test]
    fn rejects_a_riff_file_that_is_not_webp() {
        let wav = b"RIFF\x24\x00\x00\x00WAVEfmt ";
        let actual = decoder().decode(wav).unwrap_err();
        assert!(matches!(actual, ImageError::UnknownFormat));
    }

    #[test]
    fn a_truncated_webp_reports_an_error() {
        let truncated = &RGBA_WEBP[..RGBA_WEBP.len() - 4];
        assert!(decoder().decode(truncated).is_err());
    }

    #[test]
    fn a_webp_with_a_valid_header_and_a_corrupt_body_reports_an_error() {
        // Scramble the middle of the VP8L payload. The trailing bytes are padding
        // and the RIFF size field, which libwebp does not validate tightly, so
        // damaging those would decode fine.
        let mut corrupt = RGBA_WEBP.to_vec();
        let middle = corrupt.len() / 2;
        corrupt[middle] ^= 0xFF;
        corrupt[middle + 1] ^= 0xFF;
        corrupt[middle + 2] ^= 0xFF;
        assert!(decoder().decode(&corrupt).is_err());
    }

    #[test]
    fn decoding_the_same_bytes_twice_gives_the_same_image() {
        let decoder = decoder();
        let first = decoder.decode(RGBA_WEBP).unwrap();
        let second = decoder.decode(RGBA_WEBP).unwrap();
        test_utils::assert_slice_eq(first.as_slice(), second.as_slice());
    }

    #[test]
    fn names_itself_webp() {
        let decoder = decoder();
        assert_eq!(decoder.format(), ImageFormat::WebP);
        assert_eq!(decoder.name(), "webp");
    }
}
