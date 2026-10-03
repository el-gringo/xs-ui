use std::io::Read;

use xs_image_core::{
    ColorSpace, DecoderRegistry, ImageBuffer, ImageDimensions, ImageError, ImageFormat, ImageResult,
};

use crate::registry;

/// How many bytes [`Image::read_from`] will read.
///
/// Far more than a UI texture and low enough that a hostile url cannot exhaust a
/// TV's memory.
pub const MAX_READ_BYTES: u64 = 64 * 1024 * 1024;

/// How to read an image.
///
/// Deliberately small. The one knob that matters is the colour space every decoder
/// is asked to normalise to, because that is what the renderer uploads;
/// everything else the codecs work out for themselves.
///
/// # Examples
///
/// ```
/// use xs_image::{ColorSpace, Image, ImageOptions};
///
/// // Every format, whatever it natively produces.
/// let _ = Image::read(b"", ImageOptions::default());
/// // Force RGBA.
/// let _ = Image::read(b"", ImageOptions::default().color_space(ColorSpace::Rgba));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ImageOptions {
    color_space: Option<ColorSpace>,
}

impl ImageOptions {
    /// Asks every decoder to normalise to `color_space`.
    ///
    /// The default leaves each codec's natural output alone: grayscale stays
    /// grayscale for JPEG and PNG, so an opaque image costs three channels or one
    /// rather than always four.
    #[must_use]
    pub const fn color_space(mut self, color_space: ColorSpace) -> Self {
        self.color_space = Some(color_space);
        self
    }
}

/// A decoded image, with the format it came from.
///
/// The shape `zune-image` uses: `read` sniffs, `convert_color` normalises, and the
/// samples are reachable as a slice.
///
/// # Examples
///
/// ```no_run
/// use xs_image::{ColorSpace, Image, ImageOptions};
///
/// let bytes = std::fs::read("poster.jpg")?;
/// let mut image = Image::read(bytes.as_slice(), ImageOptions::default())?;
/// assert_eq!(image.format(), Some(xs_image::ImageFormat::Jpeg));
///
/// if image.color_space() != ColorSpace::Rgba {
///     image.convert_color(ColorSpace::Rgba)?;
/// }
/// let samples = image.samples();
/// # let _ = samples;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Image {
    buffer: ImageBuffer,
    format: Option<ImageFormat>,
}

impl Image {
    /// Decodes `data`, choosing the codec from the leading bytes and the registry
    /// this thread already has.
    ///
    /// # Errors
    ///
    /// [`ImageError::UnknownFormat`] when nothing claims the bytes,
    /// [`ImageError::DecoderUnavailable`] when the format is known but its library
    /// is not on this device, and whatever the codec reports otherwise.
    pub fn read(data: &[u8], options: ImageOptions) -> ImageResult<Self> {
        registry::with_registry(|registry| Image::read_with(data, registry, options))
    }

    /// Decodes `data` with an explicit registry, for a caller that wants a
    /// different codec set than this thread's.
    ///
    /// # Errors
    ///
    /// The same as [`Image::read`].
    pub fn read_with(
        data: &[u8],
        registry: &DecoderRegistry,
        options: ImageOptions,
    ) -> ImageResult<Self> {
        let format = ImageFormat::sniff(data).ok_or(ImageError::UnknownFormat)?;
        let buffer = registry.decode_as(format, data)?;
        let mut image = Self {
            buffer,
            format: Some(format),
        };
        if let Some(color_space) = options.color_space {
            image.convert_color(color_space)?;
        }
        Ok(image)
    }

    /// Reads a reader to the end and decodes it.
    ///
    /// # Errors
    ///
    /// [`ImageError::Io`] when the reader fails, plus everything [`Image::read`]
    /// reports.
    pub fn read_from(reader: &mut impl Read, options: ImageOptions) -> ImageResult<Self> {
        let bytes = registry::read_all(reader, MAX_READ_BYTES)?;
        Self::read(&bytes, options)
    }

    /// The format the bytes were identified as, or `None` for an image built by
    /// hand.
    #[must_use]
    pub const fn format(&self) -> Option<ImageFormat> {
        self.format
    }

    /// Width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.buffer.width()
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.buffer.height()
    }

    /// `(width, height)` in pixels.
    #[must_use]
    pub const fn dimensions(&self) -> (u32, u32) {
        self.buffer.dimensions()
    }

    /// How the samples are laid out.
    #[must_use]
    pub const fn color_space(&self) -> ColorSpace {
        self.buffer.color_space()
    }

    /// The raw samples.
    #[must_use]
    pub fn samples(&self) -> &[u8] {
        self.buffer.as_slice()
    }

    /// The samples, for a caller about to overwrite them.
    #[must_use]
    pub fn samples_mut(&mut self) -> &mut [u8] {
        self.buffer.as_mut_slice()
    }

    /// The underlying buffer.
    #[must_use]
    pub const fn buffer(&self) -> &ImageBuffer {
        &self.buffer
    }

    /// Consumes the image and hands back the buffer.
    #[must_use]
    pub fn into_buffer(self) -> ImageBuffer {
        self.buffer
    }

    /// Normalises the samples to `color_space`.
    ///
    /// Only RGBA is a target: it is the one every renderer can upload directly.
    ///
    /// # Errors
    ///
    /// [`ImageError::UnsupportedConversion`] for any other target, and
    /// [`ImageError::DimensionsOverflow`] when the RGBA buffer would not fit.
    pub fn convert_color(&mut self, color_space: ColorSpace) -> ImageResult<()> {
        if self.buffer.color_space() == color_space {
            return Ok(());
        }
        self.buffer = match color_space {
            ColorSpace::Rgba => self.buffer.to_rgba8()?,
            _ => {
                return Err(ImageError::UnsupportedConversion(
                    self.buffer.color_space(),
                    color_space,
                ));
            }
        };
        Ok(())
    }

    /// Reads only the header, without decoding pixels.
    ///
    /// Takes no [`ImageOptions`] because none of them apply: this allocates no
    /// pixels, so there is nothing to normalise.
    ///
    /// # Errors
    ///
    /// The same as [`Image::read`], except no pixels are allocated.
    pub fn read_dimensions(data: &[u8]) -> ImageResult<ImageDimensions> {
        registry::with_registry(|registry| {
            let format = ImageFormat::sniff(data).ok_or(ImageError::UnknownFormat)?;
            registry
                .get(format)
                .ok_or(ImageError::DecoderUnavailable(format))?
                .read_dimensions(data)
        })
    }
}
