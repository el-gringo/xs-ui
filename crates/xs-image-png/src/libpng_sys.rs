//! Raw `libpng` bindings.
//!
//! Loaded at runtime with `dlopen`, following `xs-egl-sys` and `xs-tls`.
//!
//! libpng keeps its state behind two opaque pointers, so there is nothing here to
//! keep in sync with `png.h` beyond the constants. What does need care is error
//! handling: libpng reports a hard error through `longjmp`, and calls `abort()`
//! when that jump target is missing, so [`PngReader`](crate::PngReader) installs
//! both the jump buffer and the error callbacks.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::os::raw::{c_char, c_int, c_void};

/// `size_t`.
pub type size_t = usize;
/// `png_uint_32`.
pub type png_uint_32 = u32;
/// `png_structp`, opaque.
pub type png_structp = *mut c_void;
/// `png_infop`, opaque.
pub type png_infop = *mut c_void;

/// `png_error_ptr`: called on a hard error and on a warning.
pub type png_error_ptr = Option<unsafe extern "C" fn(png_structp, *const c_char)>;
/// `png_rw_ptr`: our reader callback.
pub type png_rw_ptr = Option<unsafe extern "C" fn(png_structp, *mut u8, size_t)>;
/// `png_longjmp_ptr`.
pub type png_longjmp_ptr = Option<unsafe extern "C" fn(*mut c_void, c_int) -> !>;

/// glibc's `jmp_buf`, i.e. `struct __jmp_buf_tag`: 64 bytes of jump buffer, 8 of
/// saved-mask flag and padding, then a 128 byte `sigset_t`. Pinned by a test
/// below, and handed to `png_set_longjmp_fn` so libpng can size-check it.
pub const JMP_BUF_BYTES: usize = 200;

unsafe extern "C" {
    /// `setjmp` without the signal mask save, paired with `_longjmp`.
    pub unsafe fn _setjmp(env: *mut c_void) -> c_int;
    /// Never returns: unwinds to the matching `_setjmp`.
    pub unsafe fn _longjmp(env: *mut c_void, value: c_int) -> !;
}

/// `PNG_COLOR_TYPE_*`, pinned because a shifted value would make a decoder label
/// a palette or grayscale image as the wrong colour space.
pub const PNG_COLOR_TYPE_GRAY: c_int = 0;
pub const PNG_COLOR_TYPE_RGB: c_int = 2;
pub const PNG_COLOR_TYPE_PALETTE: c_int = 3;
pub const PNG_COLOR_TYPE_GRAY_ALPHA: c_int = 4;
pub const PNG_COLOR_TYPE_RGB_ALPHA: c_int = 6;

/// The eight bytes at the start of every PNG.
pub const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// `PNG_LIBPNG_VER_STRING` for the 1.6 series.
///
/// `png_create_read_struct_2` refuses to build a context when the caller's idea
/// of the ABI disagrees with the library's, so this is what keeps a 1.5 or 1.7
/// build from being driven with 1.6 semantics. The patch level is ignored.
pub const PNG_LIBPNG_VER: &[u8] = b"1.6.0\0";

// `png_set_longjmp_fn` hands libpng our `longjmp` and validates the buffer size
// against its own, returning null on an ABI mismatch. It is what the `png_jmpbuf`
// macro expands to, and the reason `PngReader` stores the jump buffer itself.
xs_platform_linux::declare_module! {
    LibPng,
    "libpng16.so",
    "libpng16.so.16",
    "libpng.so",
    "libpng.so.3",
    ...
    ...
    pub fn png_create_read_struct_2(
        *const c_char,
        *mut c_void,
        png_error_ptr,
        png_error_ptr,
        *mut c_void,
        *mut c_void,
        *mut c_void,
    ) -> png_structp,
    pub fn png_create_info_struct(png_structp) -> png_infop,
    pub fn png_destroy_read_struct(*mut png_structp, *mut png_infop, *mut png_infop),
    pub fn png_set_read_fn(png_structp, *mut c_void, png_rw_ptr),
    pub fn png_get_io_ptr(png_structp) -> *mut c_void,
    pub fn png_sig_cmp(*const u8, size_t, size_t) -> c_int,

    pub fn png_set_longjmp_fn(png_structp, png_longjmp_ptr, size_t) -> *mut c_void,

    pub fn png_read_info(png_structp, png_infop),
    pub fn png_read_update_info(png_structp, png_infop),
    pub fn png_read_image(png_structp, *mut *mut u8),
    pub fn png_read_end(png_structp, png_infop),
    pub fn png_get_image_width(png_structp, png_infop) -> png_uint_32,
    pub fn png_get_image_height(png_structp, png_infop) -> png_uint_32,
    pub fn png_get_rowbytes(png_structp, png_infop) -> size_t,
    pub fn png_get_channels(png_structp, png_infop) -> u8,
    pub fn png_get_bit_depth(png_structp, png_infop) -> u8,

    pub fn png_set_palette_to_rgb(png_structp),
    pub fn png_set_expand_gray_1_2_4_to_8(png_structp),
    pub fn png_set_tRNS_to_alpha(png_structp),
    pub fn png_set_packing(png_structp),
    pub fn png_set_strip_16(png_structp),
    pub fn png_set_interlace_handling(png_structp) -> c_int,
    ...
    ...
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_signature_is_the_iso_15948_one() {
        let actual = &PNG_SIGNATURE;
        let expected = &[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        test_utils::assert_slice_eq(expected, actual);
    }

    #[test]
    fn the_version_string_is_null_terminated() {
        // libpng compares the string as a C one.
        let actual = PNG_LIBPNG_VER.last().copied();
        let expected = Some(0);
        assert_eq!(actual, expected);
    }

    #[test]
    fn jmp_buf_is_glibc_s_size() {
        let actual = size_of::<[u64; 25]>();
        assert_eq!(actual, JMP_BUF_BYTES);
    }

    #[test]
    fn the_colour_types_match_the_header() {
        // Pinned because the decoder maps the transformed channel count back onto
        // a ColorSpace, and a shifted enum here would silently mislabel pixels.
        let expected = [0, 2, 3, 4, 6];
        let actual = [
            PNG_COLOR_TYPE_GRAY,
            PNG_COLOR_TYPE_RGB,
            PNG_COLOR_TYPE_PALETTE,
            PNG_COLOR_TYPE_GRAY_ALPHA,
            PNG_COLOR_TYPE_RGB_ALPHA,
        ];
        assert_eq!(actual, expected);
    }
}
