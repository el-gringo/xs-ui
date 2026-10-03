use std::{mem, os::raw::c_int};

use xs_image_core::{
    ColorSpace, ImageBuffer, ImageDecoder, ImageDimensions, ImageError, ImageFormat, ImageResult,
};

use crate::libjpeg_sys::{
    _setjmp, JCS_CMYK, JCS_GRAYSCALE, JCS_RGB, JCS_YCCK, JCS_YCbCr, JPEG_HEADER_OK,
    JPEG_LIB_VERSION, JSAMPROW, JpegErrorManager, LibJpeg, SIZEOF_JPEG_DECOMPRESS_STRUCT,
    jpeg_decompress_struct,
};

/// Headroom on the bounce row, because libjpeg may hand back a scanline padded
/// out to the right edge of an MCU rather than exactly `output_width` samples.
const SCANLINE_PADDING: usize = 128;

/// Decodes JPEG, JFIF and EXIF files with `libjpeg`.
///
/// Grayscale input stays grayscale and everything else becomes RGB: JPEG has no
/// alpha channel, so a fourth channel would only make the texture upload a
/// quarter larger.
///
/// # Unsupported
///
/// CMYK and YCCK files are rejected. libjpeg hands those back as CMYK, and
/// converting them to RGB properly needs the Adobe APP14 transform plus an ICC
/// rendering intent, which is more machinery than a TV UI is going to ship.
///
/// # Panics
///
/// [`JpegDecoder::new`] panics when `libjpeg.so` cannot be loaded; use
/// [`JpegDecoder::try_new`] to handle that. Decode failures are reported as
/// [`ImageError`] instead: libjpeg calls `exit()` by default and
/// [`JpegErrorManager`] is what stops it from doing that.
///
/// # Examples
///
/// ```
/// use xs_image_core::ImageDecoder;
/// use xs_image_jpeg::JpegDecoder;
///
/// let decoder = JpegDecoder::new();
/// let image = decoder.decode(include_bytes!("../testdata/rgb.jpg"))?;
/// assert_eq!(image.dimensions(), (2, 2));
/// # Ok::<(), xs_image_core::ImageError>(())
/// ```
#[derive(Clone)]
pub struct JpegDecoder {
    jpeg: LibJpeg,
}

impl JpegDecoder {
    /// Loads `libjpeg.so`.
    ///
    /// # Panics
    ///
    /// Panics when the shared library or any of its symbols is missing.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("failed to load libjpeg.so")
    }

    /// Loads `libjpeg.so`, reporting failure instead of panicking.
    ///
    /// # Errors
    ///
    /// [`ImageError::DecoderUnavailable`] when the library is not on this device.
    pub fn try_new() -> ImageResult<Self> {
        LibJpeg::try_load()
            .map(|jpeg| Self { jpeg })
            .map_err(|()| ImageError::DecoderUnavailable(ImageFormat::Jpeg))
    }

    /// Runs `body` against a fresh decompressor.
    ///
    /// libjpeg reports failure by longjmp-ing out of `error_exit`, and it frees
    /// the decompressor on the way out, so the whole decode has to live inside
    /// one function: nothing survives a longjmp. That is also why `data` stays
    /// borrowed for the entire body, because `jpeg_mem_src` keeps a pointer to it.
    fn with_decompressor<T>(
        &self,
        data: &[u8],
        body: impl FnOnce(&LibJpeg, *mut jpeg_decompress_struct) -> ImageResult<T>,
    ) -> ImageResult<T> {
        let mut manager = JpegErrorManager::new(&self.jpeg);
        let mut decompress = unsafe { mem::zeroed::<jpeg_decompress_struct>() };
        manager.install(&raw mut decompress);

        // `_setjmp` returns twice: 0 here, and 1 when `error_exit` longjmps back.
        // Nothing between here and that jump may move `manager`, `decompress` or
        // `data`, which is why the decode is one function and that state is only
        // ever borrowed.
        if unsafe { _setjmp(manager.jump_buffer()) } != 0 {
            // libjpeg already ran `jpeg_destroy`, so there is nothing to free.
            return Err(jump_error(&manager));
        }

        let result = unsafe {
            (self.jpeg.jpeg_CreateDecompress)(
                &raw mut decompress,
                JPEG_LIB_VERSION,
                SIZEOF_JPEG_DECOMPRESS_STRUCT,
            );
            (self.jpeg.jpeg_mem_src)(&raw mut decompress, data.as_ptr(), data.len());
            body(&self.jpeg, &raw mut decompress)
        };

        unsafe { (self.jpeg.jpeg_destroy_decompress)(&raw mut decompress) };
        result
    }

    /// Reads the frame header, filling in the requested output colour space and
    /// the dimensions on the way.
    ///
    /// Takes the raw decompressor so the callers, which own the `setjmp`
    /// contract, are the only place holding a pointer.
    fn read_header(
        jpeg: &LibJpeg,
        decompress: *mut jpeg_decompress_struct,
    ) -> ImageResult<(ImageDimensions, ColorSpace)> {
        unsafe {
            if (jpeg.jpeg_read_header)(decompress, 1) != JPEG_HEADER_OK {
                return Err(ImageError::InvalidData("no image header".into()));
            }
            let color_space = output_color_space((*decompress).jpeg_color_space)?;
            (*decompress).out_color_space = match color_space {
                ColorSpace::Luma => JCS_GRAYSCALE,
                _ => JCS_RGB,
            };
            (jpeg.jpeg_calc_output_dimensions)(decompress);

            let dimensions = ImageDimensions {
                width: (*decompress).output_width,
                height: (*decompress).output_height,
            };
            if dimensions.width == 0 || dimensions.height == 0 {
                return Err(ImageError::ZeroSized);
            }
            Ok((dimensions, color_space))
        }
    }

    /// Drains the scanlines into a fresh buffer. `dimensions` and `color_space`
    /// must be the pair `read_header` returned for this decompressor.
    fn read_pixels(
        jpeg: &LibJpeg,
        decompress: *mut jpeg_decompress_struct,
        dimensions: ImageDimensions,
        color_space: ColorSpace,
    ) -> ImageResult<ImageBuffer> {
        unsafe {
            if (jpeg.jpeg_start_decompress)(decompress) != 1 {
                return Err(ImageError::InvalidData("libjpeg refused to start".into()));
            }

            let ImageDimensions { width, height } = dimensions;
            let mut image = ImageBuffer::try_new(width, height, color_space)?;
            let row_bytes = width as usize * color_space.channels();

            let mut scanline = vec![0u8; row_bytes + SCANLINE_PADDING];
            let row_pointer: JSAMPROW = scanline.as_mut_ptr();
            let mut rows = [row_pointer];

            let mut row = 0;
            while row < height {
                if (jpeg.jpeg_read_scanlines)(decompress, rows.as_mut_ptr(), 1) != 1u32 {
                    break;
                }
                image.as_mut_slice()[row as usize * row_bytes..(row as usize + 1) * row_bytes]
                    .copy_from_slice(&scanline[..row_bytes]);
                row += 1;
            }
            (jpeg.jpeg_finish_decompress)(decompress);
            if row != height {
                return Err(ImageError::Truncated);
            }
            Ok(image)
        }
    }
}

impl Default for JpegDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDecoder for JpegDecoder {
    fn format(&self) -> ImageFormat {
        ImageFormat::Jpeg
    }

    fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions> {
        if !data.starts_with(ImageFormat::Jpeg.signature()) {
            return Err(ImageError::UnknownFormat);
        }
        self.with_decompressor(data, |jpeg, decompress| {
            Ok(JpegDecoder::read_header(jpeg, decompress)?.0)
        })
    }

    fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer> {
        if !data.starts_with(ImageFormat::Jpeg.signature()) {
            return Err(ImageError::UnknownFormat);
        }
        self.with_decompressor(data, |jpeg, decompress| {
            let (dimensions, color_space) = JpegDecoder::read_header(jpeg, decompress)?;
            JpegDecoder::read_pixels(jpeg, decompress, dimensions, color_space)
        })
    }
}

/// Picks the colour space libjpeg should convert into, keeping grayscale as
/// grayscale so a monochrome texture costs a third of the memory.
fn output_color_space(input: c_int) -> ImageResult<ColorSpace> {
    match input {
        JCS_GRAYSCALE => Ok(ColorSpace::Luma),
        JCS_RGB | JCS_YCbCr => Ok(ColorSpace::Rgb),
        JCS_CMYK | JCS_YCCK => Err(ImageError::InvalidData("cmyk jpeg is not supported".into())),
        other => Err(ImageError::InvalidData(format!(
            "unknown jpeg color space {other}"
        ))),
    }
}

/// Turns whatever `error_exit` captured into an error, keeping libjpeg's own
/// message when it produced one.
fn jump_error(manager: &JpegErrorManager) -> ImageError {
    let message = manager.message();
    if message.is_empty() {
        ImageError::InvalidData("libjpeg reported an error".into())
    } else {
        ImageError::InvalidData(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RGB_JPEG: &[u8] = include_bytes!("../testdata/rgb.jpg");
    const GRAYSCALE_JPEG: &[u8] = include_bytes!("../testdata/grayscale.jpg");

    /// The four colours baked into every fixture.
    const RED: [u8; 3] = [255, 0, 0];
    const GREEN: [u8; 3] = [0, 255, 0];
    const BLUE: [u8; 3] = [0, 0, 255];
    const WHITE: [u8; 3] = [255, 255, 255];

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------
    fn decoder() -> JpegDecoder {
        JpegDecoder::try_new().expect("libjpeg.so is not installed")
    }

    /// The fixture pixels as one flat slice, row by row.
    fn pixels(image: &ImageBuffer) -> Vec<u8> {
        image.as_slice().to_vec()
    }

    fn assert_close(actual: &[u8], expected: &[u8], tolerance: i32) {
        assert_eq!(
            actual.len(),
            expected.len(),
            "expected {} samples, got {}",
            expected.len(),
            actual.len()
        );
        for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            assert!(
                i32::from(*actual) - i32::from(*expected).abs() <= tolerance,
                "sample {index} is {actual}, expected {expected} +/- {tolerance}"
            );
        }
    }

    // --------------------------------------------------
    // Tests
    // --------------------------------------------------
    #[test]
    fn decodes_an_rgb_jpeg_to_three_channels() {
        let image = decoder().decode(RGB_JPEG).unwrap();
        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.color_space(), ColorSpace::Rgb);
        assert_eq!(image.len(), 2 * 2 * 3);
    }

    #[test]
    fn decodes_the_right_colours() {
        let image = decoder().decode(RGB_JPEG).unwrap();
        let expected = [RED, GREEN, BLUE, WHITE].concat();
        // JPEG is lossy even at quality 100: the YCbCr round trip rounds.
        assert_close(&pixels(&image), &expected, 2);
    }

    #[test]
    fn keeps_grayscale_as_grayscale() {
        let image = decoder().decode(GRAYSCALE_JPEG).unwrap();
        assert_eq!(image.color_space(), ColorSpace::Luma);
        assert_eq!(image.len(), 2 * 2);
        let expected = [0u8, 64, 128, 255];
        assert_close(&pixels(&image), &expected, 2);
    }

    #[test]
    fn expands_grayscale_to_rgba_on_request() {
        let image = decoder()
            .decode(GRAYSCALE_JPEG)
            .unwrap()
            .to_rgba8()
            .unwrap();
        assert_eq!(image.color_space(), ColorSpace::Rgba);
        assert_eq!(image.len(), 2 * 2 * 4);
    }

    #[test]
    fn reports_dimensions_without_decoding() {
        let actual = decoder().read_dimensions(RGB_JPEG).unwrap();
        let expected = ImageDimensions {
            width: 2,
            height: 2,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn the_decoded_size_matches_the_reported_one() {
        let decoder = decoder();
        let dimensions = decoder.read_dimensions(RGB_JPEG).unwrap();
        let image = decoder.decode(RGB_JPEG).unwrap();
        assert_eq!(image.dimensions(), dimensions.as_tuple());
    }

    #[test]
    fn rejects_bytes_that_are_not_a_jpeg() {
        let actual = decoder().decode(&[0x89, b'P', b'N', b'G']).unwrap_err();
        assert!(matches!(actual, ImageError::UnknownFormat));
    }

    #[test]
    fn a_truncated_jpeg_reports_an_error_instead_of_exiting() {
        // The point of this test is that it returns at all: libjpeg's default
        // handler calls exit(), which would take the test runner with it.
        let truncated = &RGB_JPEG[..RGB_JPEG.len() / 2];
        assert!(decoder().decode(truncated).is_err());
    }

    #[test]
    fn garbage_with_a_jpeg_signature_reports_an_error_instead_of_exiting() {
        let mut garbage = vec![0xFF, 0xD8, 0xFF];
        garbage.extend_from_slice(&[0x40; 512]);
        assert!(decoder().decode(&garbage).is_err());
    }

    #[test]
    fn decoding_the_same_bytes_twice_gives_the_same_image() {
        // A longjmp that landed on a stale stack would show up here.
        let decoder = decoder();
        let first = decoder.decode(RGB_JPEG).unwrap();
        let second = decoder.decode(RGB_JPEG).unwrap();
        assert_eq!(pixels(&first), pixels(&second));
    }

    #[test]
    fn a_failed_decode_does_not_break_the_next_one() {
        let decoder = decoder();
        let _ = decoder.decode(b"nonsense");
        let _ = decoder.decode(&RGB_JPEG[..4]);
        assert!(decoder.decode(RGB_JPEG).is_ok());
    }

    #[test]
    fn names_itself_jpeg() {
        let decoder = decoder();
        assert_eq!(decoder.format(), ImageFormat::Jpeg);
        assert_eq!(decoder.name(), "jpeg");
    }
}
