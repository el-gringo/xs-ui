//! Raw `libwebp` bindings.
//!
//! Loaded at runtime with `dlopen`, following `xs-egl-sys` and `xs-tls`.
//!
//! Only symbols verified present across libwebp 0.6, 1.0, 1.2 and 1.6 are
//! declared, because `declare_module!` unwraps every symbol it asks for and one
//! missing name fails the whole load. `WebPGetFeatures`, for instance, is
//! declared in `decode.h` but is not exported by every build.
//!
//! The allocating `WebPDecodeRGBA` is bound rather than the zero-copy
//! `WebPDecodeRGBAInto`: libjpeg's neighbours aside, `Into` grew an
//! `output_stride` parameter in 1.6 and changed its return type from `size_t` to
//! `uint8_t*`, so the zero-copy form needs a different declaration per version.
//! `WebPDecodeRGBA` and `WebPFree` have had the same signature since 0.6, which
//! is worth one extra copy of the pixels.

#![allow(non_camel_case_types, non_snake_case, dead_code)]

use std::{
    os::raw::{c_int, c_void},
    ptr::NonNull,
};

/// `size_t`.
pub type size_t = usize;

// `WebPGetDecoderVersion` packs the version as MAJOR << 16 | MINOR << 8 | PATCH,
// so 1.6.0 is 0x010600.
//
// `WebPGetInfo` validates the header and fills in the dimensions. It returns a
// boolean, not a `VP8StatusCode`: 1 is a usable header and the out parameters are
// only set on success.
//
// `WebPDecodeRGBA` returns a buffer of `width * height * 4` bytes in straight
// (not premultiplied) R, G, B, A order that libwebp allocated, or null. It has to
// go back through `WebPFree`, which is why the crate does not hand this pointer
// out.
xs_platform_linux::declare_module! {
    LibWebP,
    "libwebp.so",
    "libwebp.so.7",
    "libwebp.so.6",
    ...
    ...
    pub fn WebPGetDecoderVersion() -> c_int,
    pub fn WebPGetInfo(*const u8, size_t, *mut c_int, *mut c_int) -> c_int,
    pub fn WebPDecodeRGBA(*const u8, size_t, *mut c_int, *mut c_int) -> *mut u8,
    pub fn WebPFree(*mut c_void),
    ...
    ...
}

/// Owns a buffer libwebp allocated and hands it back on drop.
///
/// Without this every decode of a corrupt file leaks, and a TV decoding a
/// playlist of thumbnails leaks once per frame.
pub struct WebPBuffer {
    pointer: NonNull<u8>,
    free: unsafe extern "C" fn(*mut c_void),
}

impl WebPBuffer {
    /// Takes ownership of `pointer`, which must come from libwebp.
    ///
    /// # Safety
    ///
    /// `pointer` must be a non-null buffer libwebp allocated with its own
    /// allocator, and `free` must be the matching `WebPFree`.
    pub unsafe fn new(pointer: *mut u8, free: unsafe extern "C" fn(*mut c_void)) -> Option<Self> {
        NonNull::new(pointer).map(|pointer| Self { pointer, free })
    }

    /// The buffer's samples.
    #[must_use]
    pub fn as_slice(&self, len: usize) -> &[u8] {
        // SAFETY: libwebp guarantees `len` readable bytes for a successful decode.
        unsafe { std::slice::from_raw_parts(self.pointer.as_ptr(), len) }
    }
}

impl Drop for WebPBuffer {
    fn drop(&mut self) {
        unsafe { (self.free)(self.pointer.as_ptr().cast::<c_void>()) };
    }
}
