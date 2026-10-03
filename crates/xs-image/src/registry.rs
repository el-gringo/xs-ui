use std::io::Read;

use xs_image_core::{DecoderRegistry, ImageFormat, ImageResult};

/// Builds a registry holding every codec this crate was compiled with and whose
/// shared library is actually present.
///
/// A decoder whose library is missing is left out rather than registered and made
/// to fail, so decoding that format reports [`ImageError::DecoderUnavailable`]
/// while every other format keeps working. On a TV that is the difference between
/// "this one image does not show" and "the app does not start". Use
/// [`available_formats`] to find out what made it in.
///
/// Loading four libraries is not free, so build this once and reuse it.
#[must_use]
pub fn build_registry() -> DecoderRegistry {
    let mut registry = DecoderRegistry::new();

    #[cfg(feature = "jpeg")]
    registry.register(Box::new(xs_image_jpeg::JpegDecoder::new()));

    #[cfg(feature = "jpeg-xl")]
    registry.register(Box::new(xs_image_jpeg_xl::JpegXlDecoder::new()));

    #[cfg(feature = "png")]
    registry.register(Box::new(xs_image_png::PngDecoder::new()));

    #[cfg(feature = "webp")]
    registry.register(Box::new(xs_image_webp::WebPDecoder::new()));

    registry
}

thread_local! {
    /// Built on first use per thread.
    ///
    /// A thread local rather than a `OnceLock` because the `dlopen` handles the
    /// decoders hold are `Rc`-backed, so a registry is neither `Send` nor `Sync`.
    /// That matches the rest of the framework, which is one thread per window.
    static REGISTRY: DecoderRegistry = build_registry();
}

/// Runs `body` against this thread's registry.
pub fn with_registry<R>(body: impl FnOnce(&DecoderRegistry) -> R) -> R {
    REGISTRY.with(body)
}

/// Reads a reader to the end, giving up after `limit` bytes.
///
/// Every decoder works from a slice, so this is the one place the streaming case
/// is handled. An open ended `read_to_end` is how you turn a hostile url into an
/// out of memory abort, so pass a real bound.
///
/// # Errors
///
/// [`ImageError::Io`] when the reader fails.
pub fn read_all(reader: &mut impl Read, limit: u64) -> ImageResult<Vec<u8>> {
    let mut bytes = Vec::new();
    reader.take(limit).read_to_end(&mut bytes)?;
    Ok(bytes)
}

/// The formats this build can decode on this device.
///
/// Useful for an `Accept` header, or to grey out a file picker entry.
#[must_use]
pub fn available_formats() -> Vec<ImageFormat> {
    with_registry(|registry| registry.formats().collect())
}

/// Whether this build can decode `format` on this device.
#[must_use]
pub fn supports(format: ImageFormat) -> bool {
    with_registry(|registry| registry.contains(format))
}
