use std::{
    cell::Cell,
    os::raw::{c_char, c_int, c_void},
    ptr::{self, NonNull},
};

use xs_image_core::{
    ColorSpace, ImageBuffer, ImageDecoder, ImageDimensions, ImageError, ImageFormat, ImageResult,
};

use crate::libpng_sys::{
    _longjmp, _setjmp, JMP_BUF_BYTES, LibPng, PNG_LIBPNG_VER, PNG_SIGNATURE, png_infop, png_structp,
};

/// `JMP_BUF_BYTES` expressed as `u64` words, for the buffer declaration.
const JMP_BUF_WORDS: usize = JMP_BUF_BYTES / size_of::<u64>();

thread_local! {
    /// The reader the libpng callbacks report into.
    ///
    /// libpng hands a callback nothing but the context pointer, and getting from
    /// there back to the handle would mean calling `png_get_error_ptr` from
    /// inside the callback, which needs a library handle we are already inside
    /// of. The tree is single threaded, so a thread local resolves that.
    static ACTIVE: Cell<*mut PngReader> = const { Cell::new(ptr::null_mut()) };
}

/// The bytes libpng reads from, borrowed from the caller.
///
/// libpng keeps this pointer for the whole decode, so the value has to live in
/// the frame that owns the decode, not inside a helper.
pub(crate) struct SliceReader<'data> {
    data: &'data [u8],
    position: usize,
}

impl SliceReader<'_> {
    /// Copies the next slice into `out`, returning how many bytes landed there.
    /// Zero means end of input, which is how libpng is told the file ended.
    fn read(&mut self, out: &mut [u8]) -> usize {
        let remaining = self.data.len().saturating_sub(self.position);
        let count = remaining.min(out.len());
        out[..count].copy_from_slice(&self.data[self.position..self.position + count]);
        self.position += count;
        count
    }
}

/// Owns a libpng read context, its jump buffer and the message its error
/// callback writes.
///
/// libpng calls `abort()` when there is nowhere to jump to, so both the
/// `png_set_longjmp_fn` buffer and the `error_fn` callback have to be installed
/// before a single pixel is read.
pub struct PngReader {
    png: LibPng,
    context: NonNull<c_void>,
    info: NonNull<c_void>,
    /// glibc's `jmp_buf`. `png_set_longjmp_fn` checks the size against its own
    /// and refuses to install it on a mismatch, so a 32 bit buffer fails loudly
    /// here instead of corrupting the stack.
    jump_buffer: [u64; JMP_BUF_WORDS],
    /// Raw bytes, NUL terminated. Kept as bytes so reading it back needs no sign
    /// cast.
    message: [u8; 256],
}

impl PngReader {
    /// Creates a read context, registering this struct as libpng's `error_ptr`.
    ///
    /// # Errors
    ///
    /// [`ImageError::AllocationFailed`] when libpng cannot set up its context.
    pub fn new(png: &LibPng) -> ImageResult<Self> {
        let mut reader = Self {
            png: png.clone(),
            context: NonNull::dangling(),
            info: NonNull::dangling(),
            jump_buffer: [0; JMP_BUF_WORDS],
            message: [0; 256],
        };

        // The callbacks find this struct through `error_ptr`, so it has to be
        // addressable before the context exists.
        let context = unsafe {
            (png.png_create_read_struct_2)(
                PNG_LIBPNG_VER.as_ptr().cast::<c_char>(),
                (&raw mut reader).cast::<c_void>(),
                Some(error_fn),
                Some(warn_fn),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            )
        };
        let Some(context) = NonNull::new(context) else {
            return Err(ImageError::AllocationFailed);
        };
        reader.context = context;

        let jump_buffer =
            unsafe { (png.png_set_longjmp_fn)(context.as_ptr(), Some(longjmp), JMP_BUF_BYTES) };
        if jump_buffer.is_null() {
            // Wrong jmp_buf size for this libc: the ABI does not match.
            Self::destroy(png, context.as_ptr());
            return Err(ImageError::InvalidData(
                "libpng rejected our jmp_buf size".into(),
            ));
        }

        let info = unsafe { (png.png_create_info_struct)(context.as_ptr()) };
        let Some(info) = NonNull::new(info) else {
            Self::destroy(png, context.as_ptr());
            return Err(ImageError::AllocationFailed);
        };
        reader.info = info;
        Ok(reader)
    }

    /// Tears down a context whose `info` may never have existed.
    ///
    /// `context` must come from `png_create_read_struct_2` and be destroyed once;
    /// every caller holds that from `new`.
    fn destroy(png: &LibPng, context: png_structp) {
        let mut context = context;
        unsafe {
            (png.png_destroy_read_struct)(&raw mut context, ptr::null_mut(), ptr::null_mut());
        }
    }

    /// The context pointer libpng wants.
    #[must_use]
    pub fn context(&self) -> png_structp {
        self.context.as_ptr()
    }

    /// The info pointer libpng wants.
    #[must_use]
    pub fn info(&self) -> png_infop {
        self.info.as_ptr()
    }

    /// Points libpng's reader at `reader`, which has to outlive the decode.
    ///
    /// `reader` must stay alive and borrowed until `png_read_end` has run; every
    /// caller holds it on the frame that owns the decode.
    pub(crate) fn set_input(&self, reader: &mut SliceReader<'_>) {
        unsafe {
            (self.png.png_set_read_fn)(
                self.context(),
                ptr::from_mut(reader).cast::<c_void>(),
                Some(read_fn),
            );
        }
    }

    /// Installs the jump target and returns the value to branch on.
    ///
    /// Everything borrowed here must stay put and undropped until the matching
    /// longjmp lands back on this call, which is why the decode that calls this
    /// lives in one function.
    pub(crate) fn set_jump(&mut self) -> c_int {
        unsafe { _setjmp((&raw mut self.jump_buffer).cast::<c_void>()) }
    }

    /// The message the error callback captured.
    #[must_use]
    pub fn message(&self) -> String {
        let bytes: Vec<u8> = self
            .message
            .iter()
            .take_while(|byte| **byte != 0)
            .copied()
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }

    /// The captured message as an error, or a generic one when libpng said
    /// nothing.
    #[must_use]
    pub fn into_error(&self) -> ImageError {
        let message = self.message();
        if message.is_empty() {
            ImageError::InvalidData("libpng reported an error".into())
        } else {
            ImageError::InvalidData(message)
        }
    }
}

impl Drop for PngReader {
    fn drop(&mut self) {
        // A longjmp does not run destructors, so a decode that failed partway
        // through skips this and leaks the context. libpng offers no way to
        // clean up from inside the jump either; the leak is bounded by the
        // number of failed decodes, which for a TV UI is not a concern.
        Self::destroy(&self.png, self.context.as_ptr());
    }
}

/// libpng's read callback: hands it the next slice.
unsafe extern "C" fn read_fn(png: png_structp, out: *mut u8, count: usize) {
    // libpng gives `out` for `count` bytes; see `png_set_read_fn`.
    let Some(reader) = active_reader() else {
        return;
    };
    let io_ptr = unsafe { (reader.png.png_get_io_ptr)(png) };
    if io_ptr.is_null() {
        // Leaving the buffer untouched makes libpng read zeroes, which it will
        // then reject as a corrupt stream rather than crash on.
        return;
    }
    let Some(slice) = (unsafe { io_ptr.cast::<SliceReader<'static>>().as_mut() }) else {
        return;
    };
    unsafe { slice.read(std::slice::from_raw_parts_mut(out, count)) };
}

/// libpng's hard error callback: keep the message, then jump.
unsafe extern "C" fn error_fn(_png: png_structp, message: *const c_char) {
    if let Some(reader) = active_reader() {
        unsafe { copy_message(message, &mut reader.message) };
        let buffer = (&raw mut reader.jump_buffer).cast::<c_void>();
        unsafe { _longjmp(buffer, 1) };
    }
    // No reader to jump back into: libpng's own `png_longjmp` would abort, which
    // is the one thing this crate exists to avoid. Reaching here means
    // `error_fn` ran outside `with_reader`, which is a bug in this crate.
    panic!("libpng raised an error outside of a decode");
}

/// Warnings are dropped rather than printed, so a slightly malformed file from
/// the network cannot spam the process stderr.
unsafe extern "C" fn warn_fn(_png: png_structp, _message: *const c_char) {}

/// The `longjmp` libpng was handed by `png_set_longjmp_fn`.
unsafe extern "C" fn longjmp(buffer: *mut c_void, value: c_int) -> ! {
    unsafe { _longjmp(buffer, value) }
}

/// The reader for the decode in progress.
///
/// Returns a mutable borrow of a live local on the decoding frame, which is the
/// only way to reach it from a C callback that receives nothing but the libpng
/// context.
fn active_reader() -> Option<&'static mut PngReader> {
    ACTIVE
        .with(|active| NonNull::new(active.get()))
        .map(|reader| unsafe { &mut *reader.as_ptr() })
}

/// Copies a NUL terminated C string into a fixed buffer.
unsafe fn copy_message(source: *const c_char, destination: &mut [u8]) {
    if source.is_null() {
        return;
    }
    let bytes = unsafe { std::slice::from_raw_parts(source.cast::<u8>(), destination.len()) };
    for (slot, byte) in destination.iter_mut().zip(bytes) {
        *slot = *byte;
        if *byte == 0 {
            return;
        }
    }
}

/// Decodes PNG files with `libpng`.
///
/// Every image comes out 8 bits per sample: palettes, `tRNS` transparency and
/// sub-byte grayscale are widened, and 16 bit samples are narrowed, so the colour
/// space only depends on the channel count libpng reports. Alpha is kept when the
/// file has it rather than being forced on, which is the same rule this workspace
/// applies to JPEG.
///
/// # Panics
///
/// [`PngDecoder::new`] panics when `libpng.so` cannot be loaded; use
/// [`PngDecoder::try_new`] to handle that.
///
/// # Examples
///
/// ```
/// use xs_image_core::{ColorSpace, ImageDecoder};
/// use xs_image_png::PngDecoder;
///
/// let decoder = PngDecoder::new();
/// let image = decoder.decode(include_bytes!("../testdata/rgba.png"))?;
/// assert_eq!(image.dimensions(), (2, 2));
/// assert_eq!(image.color_space(), ColorSpace::Rgba);
/// # Ok::<(), xs_image_core::ImageError>(())
/// ```
#[derive(Clone)]
pub struct PngDecoder {
    png: LibPng,
}

impl PngDecoder {
    /// Loads `libpng.so`.
    ///
    /// # Panics
    ///
    /// Panics when the shared library or any of its symbols is missing.
    #[must_use]
    pub fn new() -> Self {
        Self::try_new().expect("failed to load libpng.so")
    }

    /// Loads `libpng.so`, reporting failure instead of panicking.
    ///
    /// # Errors
    ///
    /// [`ImageError::DecoderUnavailable`] when the library is not on this device.
    pub fn try_new() -> ImageResult<Self> {
        LibPng::try_load()
            .map(|png| Self { png })
            .map_err(|()| ImageError::DecoderUnavailable(ImageFormat::Png))
    }

    /// Runs `body` against a fresh read context over `data`.
    ///
    /// libpng reports failure by longjmp-ing out of `error_fn`, and it keeps
    /// pointers to the reader and to `data` for the whole call, so the decode has
    /// to live inside one function.
    fn with_reader<T>(
        &self,
        data: &[u8],
        body: impl FnOnce(&LibPng, &mut PngReader) -> ImageResult<T>,
    ) -> ImageResult<T> {
        let mut reader = PngReader::new(&self.png)?;
        let mut input = SliceReader { data, position: 0 };
        reader.set_input(&mut input);

        ACTIVE.with(|active| active.set(&raw mut reader));
        // `_setjmp` returns twice: 0 here, and 1 when `error_fn` longjmps back.
        // Neither `reader` nor `input` may move between here and that jump.
        let jumped = reader.set_jump();
        let result = if jumped != 0 {
            Err(reader.into_error())
        } else {
            body(&self.png, &mut reader)
        };
        // A longjmp skips this function's Drop impls, so the thread local is
        // cleared by hand on both paths rather than through a guard.
        ACTIVE.with(|active| active.set(ptr::null_mut()));
        // Ends `input`'s lifetime, which libpng still points at. The type has no
        // destructor, so this is only about telling the reader the borrow is over.
        #[allow(clippy::drop_non_drop)]
        drop(input);
        result
    }
}

impl Default for PngDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl ImageDecoder for PngDecoder {
    fn format(&self) -> ImageFormat {
        ImageFormat::Png
    }

    fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions> {
        if self.sig_cmp(data) != 0 {
            return Err(ImageError::UnknownFormat);
        }
        self.with_reader(data, |png, reader| {
            let (context, info) = (reader.context(), reader.info());
            // SAFETY: both pointers come from the live reader on this frame.
            unsafe {
                (png.png_read_info)(context, info);
                let dimensions = ImageDimensions {
                    width: (png.png_get_image_width)(context, info),
                    height: (png.png_get_image_height)(context, info),
                };
                if dimensions.width == 0 || dimensions.height == 0 {
                    return Err(ImageError::ZeroSized);
                }
                Ok(dimensions)
            }
        })
    }

    fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer> {
        if self.sig_cmp(data) != 0 {
            return Err(ImageError::UnknownFormat);
        }
        self.with_reader(data, PngDecoder::read_pixels)
    }
}

impl PngDecoder {
    /// Asks libpng whether `data` starts with the PNG signature.
    ///
    /// libpng wants a length rather than a slice comparison, since the bytes may
    /// be the head of a longer stream.
    fn sig_cmp(&self, data: &[u8]) -> c_int {
        if data.len() < PNG_SIGNATURE.len() {
            return 1;
        }
        unsafe { (self.png.png_sig_cmp)(data.as_ptr(), 0, PNG_SIGNATURE.len()) }
    }

    /// Applies the transforms that make every PNG 8 bits per sample and reads the
    /// pixels out.
    ///
    /// # Safety
    ///
    /// Only called from inside `with_reader`.
    fn read_pixels(png: &LibPng, reader: &mut PngReader) -> ImageResult<ImageBuffer> {
        let (context, info) = (reader.context(), reader.info());
        // SAFETY: `context` and `info` come from the live `PngReader` on the
        // decoding frame, and libpng's read context is valid until it is destroyed.
        unsafe {
            (png.png_read_info)(context, info);

            let width = (png.png_get_image_width)(context, info);
            let height = (png.png_get_image_height)(context, info);
            if width == 0 || height == 0 {
                return Err(ImageError::ZeroSized);
            }

            // Widen everything to 8 bit samples: sub-byte grayscale and packing,
            // palettes to RGB, tRNS to a real alpha channel, 16 bit down to 8.
            (png.png_set_palette_to_rgb)(context);
            (png.png_set_expand_gray_1_2_4_to_8)(context);
            (png.png_set_tRNS_to_alpha)(context);
            (png.png_set_packing)(context);
            (png.png_set_strip_16)(context);
            // Required before `png_read_update_info` for interlaced files.
            (png.png_set_interlace_handling)(context);
            (png.png_read_update_info)(context, info);

            let channels = (png.png_get_channels)(context, info);
            let color_space = ColorSpace::from_channels(usize::from(channels)).ok_or(
                ImageError::InvalidData(format!("libpng reported {channels} channels")),
            )?;
            if (png.png_get_bit_depth)(context, info) != 8 {
                return Err(ImageError::InvalidData(
                    "png is not 8 bits per sample after expansion".into(),
                ));
            }

            let mut image = ImageBuffer::try_new(width, height, color_space)?;
            let row_bytes = (png.png_get_rowbytes)(context, info);
            if row_bytes > image.row(0).len() {
                return Err(ImageError::DimensionsOverflow);
            }

            // `png_read_image` wants one row pointer per row. The pointers have to
            // point into the buffer, and libpng may write whole padded rows, so
            // borrow the image once and derive every pointer from it.
            let stride = image.row(0).len();
            let samples = image.as_mut_slice();
            let mut rows: Vec<*mut u8> = (0..height as usize)
                .map(|row| samples.as_mut_ptr().add(row * stride))
                .collect();
            (png.png_read_image)(context, rows.as_mut_ptr());
            (png.png_read_end)(context, ptr::null_mut());

            // libpng writes whole padded rows, so clear the slack between what it
            // wrote and what the buffer claims to hold.
            if row_bytes < stride {
                for row in 0..height as usize {
                    samples[row * stride + row_bytes..(row + 1) * stride].fill(0);
                }
            }
            Ok(image)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RGBA_PNG: &[u8] = include_bytes!("../testdata/rgba.png");
    const RGB_PNG: &[u8] = include_bytes!("../testdata/rgb.png");
    const PALETTE_PNG: &[u8] = include_bytes!("../testdata/palette.png");
    const GRAY2_PNG: &[u8] = include_bytes!("../testdata/gray2.png");
    const INTERLACED_PNG: &[u8] = include_bytes!("../testdata/interlaced.png");

    /// The four colours baked into every fixture.
    const RGB: [u8; 12] = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
    const RGBA: [u8; 16] = [
        255, 0, 0, 255, // red, opaque
        0, 255, 0, 128, // green, half transparent
        0, 0, 255, 255, // blue, opaque
        255, 255, 255, 1, // white, almost invisible
    ];

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------
    fn decoder() -> PngDecoder {
        PngDecoder::try_new().expect("libpng.so is not installed")
    }

    // --------------------------------------------------
    // Tests
    // --------------------------------------------------
    #[test]
    fn decodes_an_rgba_png_to_four_channels() {
        let image = decoder().decode(RGBA_PNG).unwrap();
        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.color_space(), ColorSpace::Rgba);
        assert_eq!(image.len(), 2 * 2 * 4);
    }

    #[test]
    fn decodes_the_right_pixels_with_alpha() {
        // PNG is lossless, so exact.
        let actual = decoder().decode(RGBA_PNG).unwrap().into_vec();
        test_utils::assert_slice_eq(&RGBA, &actual);
    }

    #[test]
    fn decodes_an_rgb_png_to_three_channels() {
        let image = decoder().decode(RGB_PNG).unwrap();
        assert_eq!(image.color_space(), ColorSpace::Rgb);
        let actual = image.into_vec();
        test_utils::assert_slice_eq(&RGB, &actual);
    }

    #[test]
    fn does_not_invent_an_alpha_channel() {
        let image = decoder().decode(RGB_PNG).unwrap();
        assert_eq!(image.len(), 2 * 2 * 3);
    }

    #[test]
    fn reports_dimensions_without_decoding() {
        let actual = decoder().read_dimensions(RGBA_PNG).unwrap();
        let expected = ImageDimensions {
            width: 2,
            height: 2,
        };
        assert_eq!(actual, expected);
    }

    #[test]
    fn the_decoded_size_matches_the_reported_one() {
        let decoder = decoder();
        let dimensions = decoder.read_dimensions(RGBA_PNG).unwrap();
        let image = decoder.decode(RGBA_PNG).unwrap();
        assert_eq!(image.dimensions(), dimensions.as_tuple());
    }

    #[test]
    fn rejects_a_jpeg() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0];
        let actual = decoder().decode(&jpeg).unwrap_err();
        assert!(matches!(actual, ImageError::UnknownFormat));
    }

    #[test]
    fn a_truncated_png_reports_an_error_instead_of_aborting() {
        // The point is that this returns: libpng's default handler calls abort().
        let truncated = &RGBA_PNG[..RGBA_PNG.len() / 2];
        assert!(decoder().decode(truncated).is_err());
    }

    #[test]
    fn garbage_after_a_valid_signature_reports_an_error() {
        let mut garbage = PNG_SIGNATURE.to_vec();
        garbage.extend_from_slice(&[0x41; 256]);
        assert!(decoder().decode(&garbage).is_err());
    }

    #[test]
    fn a_failed_decode_does_not_break_the_next_one() {
        let decoder = decoder();
        let _ = decoder.decode(&RGBA_PNG[..20]);
        let _ = decoder.decode(b"not a png at all");
        assert!(decoder.decode(RGBA_PNG).is_ok());
    }

    #[test]
    fn decoding_the_same_bytes_twice_gives_the_same_image() {
        let decoder = decoder();
        let first = decoder.decode(RGBA_PNG).unwrap();
        let second = decoder.decode(RGBA_PNG).unwrap();
        test_utils::assert_slice_eq(first.as_slice(), second.as_slice());
    }

    // --------------------------------------------------
    // Transforms
    // --------------------------------------------------
    #[test]
    fn a_palette_with_transparency_becomes_rgba() {
        // 2 bit indices, so this needs png_set_packing as well as
        // png_set_palette_to_rgb and png_set_tRNS_to_alpha.
        let image = decoder().decode(PALETTE_PNG).unwrap();
        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.color_space(), ColorSpace::Rgba);
        let actual = image.into_vec();
        test_utils::assert_slice_eq(&RGBA, &actual);
    }

    #[test]
    fn sub_byte_grayscale_widens_to_one_byte_per_sample() {
        // A 2 bit sample has to become 0, 85, 170 or 255, so every sample has to
        // land on a multiple of 85 and all four levels have to show up.
        let image = decoder().decode(GRAY2_PNG).unwrap();
        assert_eq!(image.dimensions(), (2, 2));
        assert_eq!(image.color_space(), ColorSpace::Luma);
        let samples = image.into_vec();
        assert_eq!(samples.len(), 4);
        for sample in &samples {
            assert_eq!(sample % 85, 0, "{sample} is not a widened 2 bit sample");
        }
        let mut levels = samples.clone();
        levels.sort_unstable();
        levels.dedup();
        let expected = vec![0, 85, 170, 255];
        test_utils::assert_slice_eq(&expected, &levels);
    }

    #[test]
    fn an_interlaced_png_is_de_interlaced() {
        // 8x8 Adam7. The fixture is (x * 32, y * 32, 128), so a wrong pass order
        // scrambles the rows rather than merely shifting them.
        let image = decoder().decode(INTERLACED_PNG).unwrap();
        assert_eq!(image.dimensions(), (8, 8));
        assert_eq!(image.color_space(), ColorSpace::Rgb);

        let mut expected = Vec::new();
        for y in 0..8u8 {
            for x in 0..8u8 {
                expected.extend_from_slice(&[x * 32, y * 32, 128]);
            }
        }
        let actual = image.into_vec();
        test_utils::assert_slice_eq(&expected, &actual);
    }

    #[test]
    fn names_itself_png() {
        let decoder = decoder();
        assert_eq!(decoder.format(), ImageFormat::Png);
        assert_eq!(decoder.name(), "png");
    }
}
