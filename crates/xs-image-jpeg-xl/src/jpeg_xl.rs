use std::{
    mem,
    os::raw::{c_int, c_void},
    ptr::{self, NonNull},
};

use xs_image_core::{
    ColorSpace, ImageBuffer, ImageDecoder, ImageDimensions, ImageError, ImageFormat, ImageResult,
};

use crate::libjxl_sys::{
    JXL_CHANNEL_ALPHA, JXL_DEC_BASIC_INFO, JXL_DEC_COLOR_ENCODING, JXL_DEC_ERROR, JXL_DEC_FRAME,
    JXL_DEC_FULL_IMAGE, JXL_DEC_NEED_IMAGE_OUT_BUFFER, JXL_DEC_NEED_MORE_INPUT, JXL_DEC_SUCCESS,
    JXL_FALSE, JXL_NATIVE_ENDIAN, JXL_TRUE, JXL_TYPE_UINT8, JxlBasicInfo, JxlColorEncoding,
    JxlDecoder, JxlExtraChannelInfo, JxlPixelFormat, LibJxl, MINIMUM_VERSION,
};

/// The events this decoder subscribes to.
///
/// `JXL_DEC_FULL_IMAGE` is the one that matters: it is what tells us the output
/// buffer has been filled. The rest are informative and are only subscribed to
/// because `JxlDecoderSetOutputColorProfile` is only legal between
/// `COLOR_ENCODING` and the next event.
const EVENTS: i32 =
    JXL_DEC_BASIC_INFO | JXL_DEC_COLOR_ENCODING | JXL_DEC_FRAME | JXL_DEC_FULL_IMAGE;

/// Decodes JPEG XL, ISO 18181-1, with `libjxl`.
///
/// Always produces [`ColorSpace::Rgba`]: libjxl's image out buffer is filled in
/// one go and the caller picks the format, and four channels is what the renderer
/// uploads.
///
/// # Colour
///
/// The output profile is pinned to sRGB at `JXL_DEC_COLOR_ENCODING`. Without
/// that, libjxl picks its own default and an XYB-encoded file comes back with
/// shifted channels, which looks like a subtly wrong image rather than an error.
///
/// # Version
///
/// Needs libjxl 0.11 or newer. The struct layouts in [`libjxl_sys`](crate) are
/// 0.11, and `JxlColorEncoding` in particular moved between 0.10 and 0.11 in a
/// way that would decode into garbage rather than fail.
///
/// # Panics
///
/// [`JpegXlDecoder::new`] panics when `libjxl.so` is missing or too old; use
/// [`JpegXlDecoder::try_new`] to handle that.
///
/// # Examples
///
/// ```
/// use xs_image_core::{ColorSpace, ImageDecoder};
/// use xs_image_jpeg_xl::JpegXlDecoder;
///
/// let decoder = JpegXlDecoder::new();
/// let image = decoder.decode(include_bytes!("../testdata/rgba.jxl"))?;
/// assert_eq!(image.dimensions(), (2, 2));
/// assert_eq!(image.color_space(), ColorSpace::Rgba);
/// # Ok::<(), xs_image_core::ImageError>(())
/// ```
#[derive(Clone)]
pub struct JpegXlDecoder {
    jxl: LibJxl,
    version: u32,
}

impl JpegXlDecoder {
    /// Loads `libjxl.so` and checks its version.
    ///
    /// # Panics
    ///
    /// Panics when the library is missing, or older than 0.11.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("failed to load a libjxl.so of version 0.11 or newer")
    }

    /// Loads `libjxl.so`, reporting failure instead of panicking.
    ///
    /// # Errors
    ///
    /// [`ImageError::DecoderUnavailable`] when the library is not on this device
    /// or predates 0.11.
    pub fn try_new() -> ImageResult<Self> {
        let jxl =
            LibJxl::try_load().map_err(|()| ImageError::DecoderUnavailable(ImageFormat::JpegXl))?;
        let version = unsafe { (jxl.JxlDecoderVersion)() };
        if version < MINIMUM_VERSION {
            return Err(ImageError::InvalidData(format!(
                "libjxl {version} is too old, {MINIMUM_VERSION} is the minimum"
            )));
        }
        Ok(Self { jxl, version })
    }

    /// The linked libjxl version, as `MAJOR * 1000000 + MINOR * 1000 + PATCH`.
    #[must_use]
    pub const fn version(&self) -> u32 {
        self.version
    }

    /// Runs `body` against a fresh decoder over `data`.
    ///
    /// Creates a decoder, runs `body` against it and destroys it again.
    ///
    /// The closure is what sees `data`, which is what keeps the borrow alive for
    /// as long as libjxl holds its pointer into it.
    fn with_decoder<T>(
        &self,
        body: impl FnOnce(&LibJxl, *mut JxlDecoder) -> ImageResult<T>,
    ) -> ImageResult<T> {
        let decoder = unsafe { (self.jxl.JxlDecoderCreate)(ptr::null()) };
        let decoder = NonNull::new(decoder).ok_or(ImageError::AllocationFailed)?;

        let result = body(&self.jxl, decoder.as_ptr());
        unsafe { (self.jxl.JxlDecoderDestroy)(decoder.as_ptr()) };
        result
    }

    /// Whether the extra channel at `index` is an alpha channel, which is what
    /// decides whether the output gets one.
    ///
    /// # Safety
    ///
    /// Only valid between `JXL_DEC_BASIC_INFO` and the next event.
    unsafe fn has_alpha(jxl: &LibJxl, decoder: *mut JxlDecoder, count: u32) -> bool {
        (0..count as usize).any(|index| {
            let mut info = unsafe { mem::zeroed::<JxlExtraChannelInfo>() };
            let status =
                unsafe { (jxl.JxlDecoderGetExtraChannelInfo)(decoder, index, &raw mut info) };
            status == JXL_DEC_SUCCESS && info.kind == JXL_CHANNEL_ALPHA
        })
    }

    /// Drives `JxlDecoderProcessInput` until the status loop settles.
    ///
    /// Takes the raw decoder so the one caller that owns the `data` borrow is the
    /// only place holding a pointer.
    fn pump(
        jxl: &LibJxl,
        decoder: *mut JxlDecoder,
        buffer: &mut Vec<u8>,
        requested: ColorSpace,
    ) -> ImageResult<(ImageDimensions, ColorSpace)> {
        let mut state = DecodeState::new(requested);
        loop {
            let status = unsafe { (jxl.JxlDecoderProcessInput)(decoder) };
            match status {
                JXL_DEC_SUCCESS => break,
                JXL_DEC_ERROR => {
                    return Err(ImageError::InvalidData("libjxl rejected the stream".into()));
                }
                // All the input went over in one `SetInput`, so this means the
                // stream really is short. `JxlDecoderReleaseInput` is deliberately
                // never called: releasing without a matching `SetInput` leaves
                // libjxl thinking the stream ended early, and leftover bytes after
                // `FULL_IMAGE` are normal anyway, since that is where trailing
                // metadata boxes live.
                JXL_DEC_NEED_MORE_INPUT => return Err(ImageError::Truncated),
                _ => JpegXlDecoder::on_event(status, jxl, decoder, buffer, &mut state)?,
            }
        }
        if !state.have_basic_info {
            return Err(ImageError::InvalidData("empty jpeg xl stream".into()));
        }
        Ok((state.dimensions, state.color_space))
    }

    /// Acts on one informative event from the decoder.
    ///
    /// # Safety
    ///
    /// Only called from `pump`, which owns the decoder and the `data` borrow.
    fn on_event(
        status: c_int,
        jxl: &LibJxl,
        decoder: *mut JxlDecoder,
        buffer: &mut Vec<u8>,
        state: &mut DecodeState,
    ) -> ImageResult<()> {
        // SAFETY: the decoder, `buffer` and `state` all belong to `pump`.
        unsafe {
            match status {
                JXL_DEC_NEED_IMAGE_OUT_BUFFER => {
                    if !state.have_basic_info {
                        return Err(ImageError::InvalidData("no dimensions".into()));
                    }
                    let format = JxlPixelFormat {
                        // `channels` is 1 to 4, so this cannot fail.
                        num_channels: u32::try_from(state.color_space.channels()).unwrap_or(4),
                        data_type: JXL_TYPE_UINT8,
                        endianness: JXL_NATIVE_ENDIAN,
                        align: 0,
                    };
                    let mut needed = 0;
                    if (jxl.JxlDecoderImageOutBufferSize)(
                        decoder,
                        &raw const format,
                        &raw mut needed,
                    ) != JXL_DEC_SUCCESS
                    {
                        return Err(ImageError::InvalidData(
                            "libjxl cannot size the output".into(),
                        ));
                    }
                    *buffer = ImageBuffer::try_new(
                        state.dimensions.width,
                        state.dimensions.height,
                        state.color_space,
                    )?
                    .into_vec();
                    buffer.resize(needed.max(state.color_space.channels()), 0);
                    if (jxl.JxlDecoderSetImageOutBuffer)(
                        decoder,
                        &raw const format,
                        buffer.as_mut_ptr().cast::<c_void>(),
                        buffer.len(),
                    ) != JXL_DEC_SUCCESS
                    {
                        return Err(ImageError::InvalidData(
                            "libjxl refused the output buffer".into(),
                        ));
                    }
                }
                JXL_DEC_BASIC_INFO => {
                    let mut basic = mem::zeroed::<JxlBasicInfo>();
                    if (jxl.JxlDecoderGetBasicInfo)(decoder, &raw mut basic) != JXL_DEC_SUCCESS {
                        return Err(ImageError::InvalidData("no basic info".into()));
                    }
                    state.have_basic_info = true;
                    state.dimensions = ImageDimensions {
                        width: basic.xsize,
                        height: basic.ysize,
                    };
                    if state.dimensions.width == 0 || state.dimensions.height == 0 {
                        return Err(ImageError::ZeroSized);
                    }
                    // Extra channels mean an alpha sample to carry them.
                    state.color_space =
                        if JpegXlDecoder::has_alpha(jxl, decoder, basic.num_extra_channels) {
                            ColorSpace::Rgba
                        } else {
                            ColorSpace::Rgb
                        };
                }
                JXL_DEC_COLOR_ENCODING => {
                    // Without this libjxl picks its own output profile, and an XYB
                    // encoded file decodes to the wrong colours.
                    let srgb = JxlColorEncoding::srgb();
                    (jxl.JxlDecoderSetOutputColorProfile)(decoder, &raw const srgb, ptr::null(), 0);
                }
                JXL_DEC_FRAME => {
                    // An animation would emit more frames; this decoder only
                    // returns the first, so the rest are skipped.
                    if state.frame_seen {
                        (jxl.JxlDecoderSkipCurrentFrame)(decoder);
                    }
                    state.frame_seen = true;
                }
                JXL_DEC_FULL_IMAGE => {}
                other => {
                    return Err(ImageError::InvalidData(format!(
                        "unexpected libjxl status {other}"
                    )));
                }
            }
        }
        Ok(())
    }
}

/// What the decoder's status loop has learned so far.
struct DecodeState {
    have_basic_info: bool,
    frame_seen: bool,
    dimensions: ImageDimensions,
    color_space: ColorSpace,
}

impl DecodeState {
    /// Starts at the caller's requested colour space, which `JXL_DEC_BASIC_INFO`
    /// may downgrade to RGB when the file turns out to have no alpha.
    fn new(color_space: ColorSpace) -> Self {
        Self {
            have_basic_info: false,
            frame_seen: false,
            dimensions: ImageDimensions {
                width: 0,
                height: 0,
            },
            color_space,
        }
    }
}

impl Default for JpegXlDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDecoder for JpegXlDecoder {
    fn format(&self) -> ImageFormat {
        ImageFormat::JpegXl
    }

    fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions> {
        if !JpegXlDecoder::has_signature(data) {
            return Err(ImageError::UnknownFormat);
        }
        let mut buffer = Vec::new();
        self.with_decoder(|jxl, decoder| {
            // SAFETY: a fresh decoder from `with_decoder`, and `data` outlives it.
            JpegXlDecoder::setup(jxl, decoder, data)?;
            JpegXlDecoder::pump(jxl, decoder, &mut buffer, ColorSpace::Rgba)
        })
        .map(|(dimensions, _)| dimensions)
    }

    fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer> {
        if !JpegXlDecoder::has_signature(data) {
            return Err(ImageError::UnknownFormat);
        }
        let mut buffer = Vec::new();
        // Starts as RGBA and is downgraded to RGB by `pump` if the header says
        // the file has no alpha channel.
        let (dimensions, color_space) = self.with_decoder(|jxl, decoder| {
            // SAFETY: a fresh decoder from `with_decoder`, and `data` outlives it.
            JpegXlDecoder::setup(jxl, decoder, data)?;
            JpegXlDecoder::pump(jxl, decoder, &mut buffer, ColorSpace::Rgba)
        })?;

        let mut image = ImageBuffer::try_new(dimensions.width, dimensions.height, color_space)?;
        let len = image.len();
        if buffer.len() < len {
            return Err(ImageError::Truncated);
        }
        image.as_mut_slice().copy_from_slice(&buffer[..len]);
        Ok(image)
    }
}

impl JpegXlDecoder {
    /// Whether `data` starts with a codestream or container signature.
    fn has_signature(data: &[u8]) -> bool {
        // Bare codestream `FF 0A`, or the `JXL ` signature box.
        data.starts_with(&[0xFF, 0x0A])
            || data.starts_with(&[0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20])
    }

    /// Subscribes to the events and hands libjxl its input.
    ///
    /// Only valid on a fresh decoder inside `with_decoder`, and `data` has to
    /// outlive it, which the caller guarantees by holding the slice.
    fn setup(jxl: &LibJxl, decoder: *mut JxlDecoder, data: &[u8]) -> ImageResult<()> {
        // SAFETY: a fresh decoder from `with_decoder`, and `data` outlives it.
        unsafe {
            if (jxl.JxlDecoderSubscribeEvents)(decoder, EVENTS) != JXL_DEC_SUCCESS {
                return Err(ImageError::InvalidData("could not subscribe".into()));
            }
            // A single frame, with the pixel data oriented as the image says.
            (jxl.JxlDecoderSetCoalescing)(decoder, JXL_TRUE);
            (jxl.JxlDecoderSetKeepOrientation)(decoder, JXL_FALSE);
            // libjxl otherwise hands back premultiplied alpha for associated alpha.
            (jxl.JxlDecoderSetUnpremultiplyAlpha)(decoder, JXL_FALSE);
            if (jxl.JxlDecoderSetInput)(decoder, data.as_ptr(), data.len()) != JXL_DEC_SUCCESS {
                return Err(ImageError::InvalidData("could not attach input".into()));
            }
            (jxl.JxlDecoderCloseInput)(decoder);
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RGBA_JXL: &[u8] = include_bytes!("../testdata/rgba.jxl");

    /// The four colours baked into every fixture.
    const EXPECTED: [u8; 16] = [
        255, 0, 0, 255, // red, opaque
        0, 255, 0, 128, // green, half transparent
        0, 0, 255, 255, // blue, opaque
        255, 255, 255, 1, // white, almost invisible
    ];

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------
    fn decoder() -> JpegXlDecoder {
        JpegXlDecoder::try_new().expect("libjxl.so 0.11+ is not installed")
    }

    // --------------------------------------------------
    // Tests
    // --------------------------------------------------
    #[test]
    fn decodes_a_jpeg_xl_to_rgba() {
        let image = decoder().decode(RGBA_JXL).unwrap();
        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.color_space(), ColorSpace::Rgba);
        assert_eq!(image.len(), 2 * 2 * 4);
    }

    #[test]
    fn decodes_the_right_pixels() {
        // The fixture is lossless with the original profile kept, so exact.
        let actual = decoder().decode(RGBA_JXL).unwrap().into_vec();
        test_utils::assert_slice_eq(&EXPECTED, &actual);
    }

    #[test]
    fn requests_srgb_so_the_colours_are_not_shifted() {
        // Pure red decoded without an explicit output profile comes back with a
        // large blue component, so this guards the JXL_DEC_COLOR_ENCODING branch.
        let image = decoder().decode(RGBA_JXL).unwrap();
        let red = &image.as_slice()[0..3];
        let expected = [255, 0, 0];
        test_utils::assert_slice_eq(&expected, red);
    }

    #[test]
    fn reports_dimensions_without_decoding() {
        let actual = decoder().read_dimensions(RGBA_JXL).unwrap();
        let expected = ImageDimensions {
            width: 2,
            height: 2,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn reports_a_version_of_at_least_0_11() {
        assert!(decoder().version() >= MINIMUM_VERSION);
    }

    #[test]
    fn rejects_a_png() {
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let actual = decoder().decode(&png).unwrap_err();
        assert!(matches!(actual, ImageError::UnknownFormat));
    }

    #[test]
    fn a_truncated_stream_reports_an_error() {
        let truncated = &RGBA_JXL[..RGBA_JXL.len() / 2];
        assert!(decoder().decode(truncated).is_err());
    }

    #[test]
    fn garbage_after_a_valid_signature_reports_an_error() {
        let mut garbage = vec![
            0x00, 0x00, 0x00, 0x0C, 0x4A, 0x58, 0x4C, 0x20, 0x0D, 0x0A, 0x87, 0x0A,
        ];
        garbage.extend_from_slice(&[0x5A; 256]);
        assert!(decoder().decode(&garbage).is_err());
    }

    #[test]
    fn decoding_the_same_bytes_twice_gives_the_same_image() {
        let decoder = decoder();
        let first = decoder.decode(RGBA_JXL).unwrap();
        let second = decoder.decode(RGBA_JXL).unwrap();
        test_utils::assert_slice_eq(first.as_slice(), second.as_slice());
    }

    #[test]
    fn a_failed_decode_does_not_break_the_next_one() {
        let decoder = decoder();
        let _ = decoder.decode(b"not a jpeg xl at all");
        assert!(decoder.decode(RGBA_JXL).is_ok());
    }

    #[test]
    fn names_itself_jpeg_xl() {
        let decoder = decoder();
        assert_eq!(decoder.format(), ImageFormat::JpegXl);
        assert_eq!(decoder.name(), "jpeg-xl");
    }
}
