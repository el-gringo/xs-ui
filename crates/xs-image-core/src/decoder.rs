use crate::{ImageBuffer, ImageError, ImageFormat, ImageResult};

/// The size an image header declares, before any pixels exist.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ImageDimensions {
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

impl ImageDimensions {
    /// The dimensions as a tuple, for the many call sites that only want to
    /// forward them.
    #[must_use]
    pub const fn as_tuple(self) -> (u32, u32) {
        (self.width, self.height)
    }
}

/// One image codec.
///
/// Object safe on purpose: `xs-image` boxes these into a
/// [`DecoderRegistry`] and picks one by sniffing the file header, so the
/// decoder set is a runtime value rather than a match statement.
///
/// Implementations are stateless. The native library handle a decoder holds is
/// a bundle of function pointers, and all decompression state lives on the
/// stack for the duration of one call, which is what lets `decode` take
/// `&self`.
pub trait ImageDecoder {
    /// The format this decoder claims.
    fn format(&self) -> ImageFormat;

    /// Lowercase format name, for error messages.
    fn name(&self) -> &'static str {
        self.format().name()
    }

    /// Reads only the header, without decoding pixels.
    ///
    /// # Errors
    ///
    /// [`ImageError`] when the header is missing or malformed.
    fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions>;

    /// Decodes the whole image.
    ///
    /// # Errors
    ///
    /// [`ImageError`] when the data is not this format, is truncated, or is
    /// rejected by the codec.
    fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer>;
}

impl ImageDecoder for Box<dyn ImageDecoder> {
    fn format(&self) -> ImageFormat {
        (**self).format()
    }

    fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions> {
        (**self).read_dimensions(data)
    }

    fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer> {
        (**self).decode(data)
    }
}

/// The set of codecs this process has, keyed by format.
///
/// This is the dynamic dispatch seam: `xs-image` builds one registry out of
/// whichever codec crates its features pulled in, and every decode goes through
/// the same `&dyn ImageDecoder` path.
#[derive(Default)]
pub struct DecoderRegistry {
    decoders: Vec<Box<dyn ImageDecoder>>,
}

impl DecoderRegistry {
    /// An empty registry. Decoding anything fails with
    /// [`ImageError::UnknownFormat`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a decoder, replacing any existing one for the same format.
    pub fn register(&mut self, decoder: Box<dyn ImageDecoder>) {
        let format = decoder.format();
        self.decoders.retain(|entry| entry.format() != format);
        self.decoders.push(decoder);
    }

    /// Builds a registry from an iterator of boxed decoders.
    pub fn from_decoders(decoders: impl IntoIterator<Item = Box<dyn ImageDecoder>>) -> Self {
        let mut registry = Self::new();
        for decoder in decoders {
            registry.register(decoder);
        }
        registry
    }

    /// Whether a decoder is registered for `format`.
    #[must_use]
    pub fn contains(&self, format: ImageFormat) -> bool {
        self.decoders.iter().any(|entry| entry.format() == format)
    }

    /// The decoder for `format`, if there is one.
    #[must_use]
    pub fn get(&self, format: ImageFormat) -> Option<&dyn ImageDecoder> {
        self.decoders
            .iter()
            .find(|entry| entry.format() == format)
            .map(AsRef::as_ref)
    }

    /// The formats this registry can decode, in registration order.
    pub fn formats(&self) -> impl Iterator<Item = ImageFormat> + '_ {
        self.decoders.iter().map(ImageDecoder::format)
    }

    /// The decoder for the format `data` sniffs to, if there is one.
    ///
    /// `None` here covers both "no decoder was compiled in" and "this format was
    /// never compiled in at all"; [`DecoderRegistry::decode`] tells them apart.
    #[must_use]
    pub fn detect(&self, data: &[u8]) -> Option<&dyn ImageDecoder> {
        ImageFormat::sniff(data).and_then(|format| self.get(format))
    }

    /// Sniffs `data` and decodes it with the matching decoder.
    ///
    /// # Errors
    ///
    /// [`ImageError::UnknownFormat`] when nothing claims the bytes, and
    /// whatever the chosen decoder reports otherwise.
    pub fn decode(&self, data: &[u8]) -> ImageResult<ImageBuffer> {
        let format = ImageFormat::sniff(data).ok_or(ImageError::UnknownFormat)?;
        self.decode_as(format, data)
    }

    /// Decodes `data` with the decoder registered for `format`, skipping the
    /// sniff. Useful when the format is already known from a url or a
    /// `Content-Type`.
    ///
    /// # Errors
    ///
    /// [`ImageError::DecoderUnavailable`] when the format is known but has no
    /// decoder, and whatever the chosen decoder reports otherwise.
    pub fn decode_as(&self, format: ImageFormat, data: &[u8]) -> ImageResult<ImageBuffer> {
        self.get(format)
            .ok_or(ImageError::DecoderUnavailable(format))?
            .decode(data)
    }

    /// Sniffs `data` and reads its header with the matching decoder.
    ///
    /// # Errors
    ///
    /// [`ImageError::UnknownFormat`] when nothing claims the bytes, and
    /// whatever the chosen decoder reports otherwise.
    pub fn read_dimensions(&self, data: &[u8]) -> ImageResult<ImageDimensions> {
        let format = ImageFormat::sniff(data).ok_or(ImageError::UnknownFormat)?;
        self.get(format)
            .ok_or(ImageError::DecoderUnavailable(format))?
            .read_dimensions(data)
    }
}

impl std::fmt::Debug for DecoderRegistry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list()
            .entries(self.decoders.iter().map(ImageDecoder::format))
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorSpace;

    /// A decoder that only knows how to produce a fixed 1x1 grey pixel, so the
    /// registry tests do not need a native library.
    struct FakeDecoder {
        format: ImageFormat,
    }

    impl ImageDecoder for FakeDecoder {
        fn format(&self) -> ImageFormat {
            self.format
        }

        fn read_dimensions(&self, _data: &[u8]) -> ImageResult<ImageDimensions> {
            Ok(ImageDimensions {
                width: 1,
                height: 1,
            })
        }

        fn decode(&self, _data: &[u8]) -> ImageResult<ImageBuffer> {
            ImageBuffer::try_new(1, 1, ColorSpace::Rgba)
        }
    }

    // --------------------------------------------------
    // Helpers
    // --------------------------------------------------
    fn fake(format: ImageFormat) -> Box<dyn ImageDecoder> {
        Box::new(FakeDecoder { format })
    }

    fn jpeg_bytes() -> Vec<u8> {
        vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10]
    }

    // --------------------------------------------------
    // Tests
    // --------------------------------------------------
    #[test]
    fn an_empty_registry_knows_no_format() {
        let registry = DecoderRegistry::new();
        for format in ImageFormat::ALL {
            assert!(!registry.contains(format));
        }
    }

    #[test]
    fn registering_a_format_twice_replaces_the_first_decoder() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Png));
        registry.register(fake(ImageFormat::Png));
        let expected = 1;
        assert_eq!(registry.formats().count(), expected);
    }

    #[test]
    fn get_returns_the_registered_decoder() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Jpeg));
        let actual = registry.get(ImageFormat::Jpeg).map(ImageDecoder::name);
        let expected = Some("jpeg");
        assert_eq!(actual, expected);
    }

    #[test]
    fn decode_routes_on_the_sniffed_format() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Jpeg));
        let image = registry.decode(&jpeg_bytes()).unwrap();
        assert_eq!(image.dimensions(), (1, 1));
    }

    #[test]
    fn decode_rejects_bytes_no_format_claims() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Jpeg));
        let actual = registry.decode(b"not an image").unwrap_err();
        assert!(matches!(actual, ImageError::UnknownFormat));
    }

    #[test]
    fn decode_reports_a_known_format_with_no_decoder_as_unavailable() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Jpeg));
        let png = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        let actual = registry.decode(&png).unwrap_err();
        assert!(matches!(
            actual,
            ImageError::DecoderUnavailable(ImageFormat::Png)
        ));
    }

    #[test]
    fn decode_as_uses_the_given_format_without_sniffing() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Png));
        let actual = registry.decode_as(ImageFormat::Png, &[]).unwrap();
        assert_eq!(actual.dimensions(), (1, 1));
    }

    #[test]
    fn read_dimensions_routes_on_the_sniffed_format() {
        let mut registry = DecoderRegistry::new();
        registry.register(fake(ImageFormat::Jpeg));
        let actual = registry.read_dimensions(&jpeg_bytes()).unwrap();
        let expected = ImageDimensions {
            width: 1,
            height: 1,
        };
        assert_eq!(actual, expected);
        assert_eq!(actual.as_tuple(), (1, 1));
    }

    #[test]
    fn from_decoders_registers_everything_at_once() {
        let registry = DecoderRegistry::from_decoders([
            fake(ImageFormat::Jpeg),
            fake(ImageFormat::Png),
            fake(ImageFormat::WebP),
        ]);
        let actual = registry.formats().collect::<Vec<_>>();
        let expected = vec![ImageFormat::Jpeg, ImageFormat::Png, ImageFormat::WebP];
        assert_eq!(actual, expected);
    }

    #[test]
    fn a_boxed_decoder_is_itself_a_decoder() {
        let boxed: Box<dyn ImageDecoder> = fake(ImageFormat::WebP);
        let actual = boxed.format();
        let expected = ImageFormat::WebP;
        assert_eq!(actual, expected);
    }
}
