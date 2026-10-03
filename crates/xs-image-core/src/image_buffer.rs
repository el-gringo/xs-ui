use crate::{ColorSpace, ImageError};

/// A decoded image: samples in row major order, top row first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageBuffer {
    width: u32,
    height: u32,
    color_space: ColorSpace,
    data: Vec<u8>,
}

impl ImageBuffer {
    /// An all-zero buffer of `width` x `height`, alpha included, so decoders can
    /// write straight into it.
    ///
    /// # Panics
    ///
    /// Panics when the requested size cannot be allocated, or when the channel
    /// count overflows a `usize`. Callers that decode untrusted bytes should
    /// bound the dimensions with [`ImageBuffer::try_new`] first.
    #[must_use]
    pub fn new(width: u32, height: u32, color_space: ColorSpace) -> Self {
        let len = Self::byte_len(width, height, color_space).expect("invalid image dimensions");
        Self {
            width,
            height,
            color_space,
            data: vec![0; len],
        }
    }

    /// Like [`ImageBuffer::new`], but reports a pixel count or allocation that
    /// does not fit instead of panicking. Decoders must use this: the
    /// dimensions come straight out of a file header an attacker controls.
    ///
    /// # Errors
    ///
    /// [`ImageError::DimensionsOverflow`](crate::ImageError::DimensionsOverflow) when the pixel
    /// count overflows, or the allocation fails.
    pub fn try_new(width: u32, height: u32, color_space: ColorSpace) -> Result<Self, ImageError> {
        let len = Self::byte_len(width, height, color_space)?;
        let mut data = Vec::new();
        data.try_reserve_exact(len)
            .map_err(|_| ImageError::AllocationFailed)?;
        data.resize(len, 0);
        Ok(Self {
            width,
            height,
            color_space,
            data,
        })
    }

    fn byte_len(width: u32, height: u32, color_space: ColorSpace) -> Result<usize, ImageError> {
        let pixels = (width as usize)
            .checked_mul(height as usize)
            .ok_or(ImageError::DimensionsOverflow)?;
        pixels
            .checked_mul(color_space.channels())
            .ok_or(ImageError::DimensionsOverflow)
    }

    /// Width in pixels.
    #[must_use]
    pub const fn width(&self) -> u32 {
        self.width
    }

    /// Height in pixels.
    #[must_use]
    pub const fn height(&self) -> u32 {
        self.height
    }

    /// How the samples are laid out.
    #[must_use]
    pub const fn color_space(&self) -> ColorSpace {
        self.color_space
    }

    /// `(width, height)` in pixels.
    #[must_use]
    pub const fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// The raw samples.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }

    /// The raw samples, for decoders filling a buffer they already own.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    /// Takes the samples out of the buffer, leaving it zero length.
    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.data
    }

    /// Whether the buffer holds no samples.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Samples the buffer must hold for its dimensions and colour space.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Expands to [`ColorSpace::Rgba`], which is what the renderer uploads.
    ///
    /// Already-RGBA input is returned as is, so this is cheap to call
    /// unconditionally. Grayscale is broadcast across the colour channels and
    /// opaque images get a fully opaque alpha.
    ///
    /// # Errors
    ///
    /// [`ImageError::DimensionsOverflow`](crate::ImageError::DimensionsOverflow) when the RGBA
    /// buffer would overflow.
    pub fn to_rgba8(&self) -> Result<ImageBuffer, ImageError> {
        if self.color_space == ColorSpace::Rgba {
            return Ok(self.clone());
        }
        let mut out = ImageBuffer::try_new(self.width, self.height, ColorSpace::Rgba)?;
        let source = self.as_slice();
        let destination = out.as_mut_slice();
        let pixels = self.width as usize * self.height as usize;
        for pixel in 0..pixels {
            let (src, dst) = (
                pixel * self.color_space.channels(),
                pixel * ColorSpace::Rgba.channels(),
            );
            let (first, alpha) = match self.color_space {
                ColorSpace::Luma | ColorSpace::Rgb => (source[src], u8::MAX),
                ColorSpace::LumaAlpha => (source[src], source[src + 1]),
                ColorSpace::Rgba => unreachable!("returned above"),
            };
            if self.color_space == ColorSpace::Rgb {
                destination[dst] = source[src];
                destination[dst + 1] = source[src + 1];
                destination[dst + 2] = source[src + 2];
            } else {
                // Grayscale is broadcast across the colour channels.
                destination[dst] = first;
                destination[dst + 1] = first;
                destination[dst + 2] = first;
            }
            destination[dst + 3] = alpha;
        }
        Ok(out)
    }

    /// The samples of one row, or an empty slice when the row is out of range.
    #[must_use]
    pub fn row(&self, y: u32) -> &[u8] {
        let stride = self.width as usize * self.color_space.channels();
        self.data
            .get(y as usize * stride..(y as usize + 1) * stride)
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_new_buffer_is_zero_filled() {
        let actual = ImageBuffer::new(2, 3, ColorSpace::Rgba);
        let expected = vec![0u8; 2 * 3 * 4];
        test_utils::assert_slice_eq(&expected, actual.as_slice());
    }

    #[test]
    fn a_new_buffer_has_the_requested_shape() {
        let actual = ImageBuffer::new(7, 5, ColorSpace::Rgb);
        assert_eq!(actual.dimensions(), (7, 5));
        assert_eq!(actual.color_space(), ColorSpace::Rgb);
        assert_eq!(actual.len(), 7 * 5 * 3);
        assert!(!actual.is_empty());
    }

    #[test]
    fn try_new_refuses_dimensions_that_overflow() {
        let actual = ImageBuffer::try_new(u32::MAX, u32::MAX, ColorSpace::Rgba);
        assert!(matches!(actual, Err(ImageError::DimensionsOverflow)));
    }

    #[test]
    fn to_rgba8_appends_an_opaque_alpha_to_rgb() {
        let mut source = ImageBuffer::new(1, 1, ColorSpace::Rgb);
        source.as_mut_slice().copy_from_slice(&[10, 20, 30]);
        let actual = source.to_rgba8().unwrap();
        let expected = vec![10, 20, 30, 255];
        test_utils::assert_slice_eq(&expected, actual.as_slice());
        assert_eq!(actual.color_space(), ColorSpace::Rgba);
    }

    #[test]
    fn to_rgba8_broadcasts_grayscale_across_the_colour_channels() {
        let mut source = ImageBuffer::new(1, 2, ColorSpace::Luma);
        source.as_mut_slice().copy_from_slice(&[0, 128]);
        let actual = source.to_rgba8().unwrap();
        let expected = vec![0, 0, 0, 255, 128, 128, 128, 255];
        test_utils::assert_slice_eq(&expected, actual.as_slice());
    }

    #[test]
    fn to_rgba8_keeps_the_alpha_of_grayscale_alpha() {
        let mut source = ImageBuffer::new(1, 1, ColorSpace::LumaAlpha);
        source.as_mut_slice().copy_from_slice(&[64, 32]);
        let actual = source.to_rgba8().unwrap();
        let expected = vec![64, 64, 64, 32];
        test_utils::assert_slice_eq(&expected, actual.as_slice());
    }

    #[test]
    fn to_rgba8_leaves_rgba_untouched() {
        let mut source = ImageBuffer::new(1, 1, ColorSpace::Rgba);
        source.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
        let actual = source.to_rgba8().unwrap();
        let expected = vec![1, 2, 3, 4];
        test_utils::assert_slice_eq(&expected, actual.as_slice());
    }

    #[test]
    fn row_returns_one_scanline() {
        let mut source = ImageBuffer::new(2, 2, ColorSpace::Luma);
        source.as_mut_slice().copy_from_slice(&[1, 2, 3, 4]);
        assert_eq!(source.row(0), &[1, 2]);
        assert_eq!(source.row(1), &[3, 4]);
        assert_eq!(source.row(2).len(), 0);
    }

    #[test]
    fn into_vec_hands_over_the_samples() {
        let actual = ImageBuffer::new(1, 1, ColorSpace::Luma).into_vec();
        let expected = vec![0u8];
        assert_eq!(actual, expected);
    }
}
