//! Image decoding for `xs-ui`, in the shape of `zune-image`.
//!
//! One entry point sniffs the bytes and dispatches to whichever native decoder
//! claims them:
//!
//! ```no_run
//! use xs_image::{ColorSpace, Image, ImageOptions};
//!
//! let bytes = std::fs::read("poster.png")?;
//! let image = Image::read(
//!     bytes.as_slice(),
//!     ImageOptions::default().color_space(ColorSpace::Rgba),
//! )?;
//! println!("{}x{} rgba", image.width(), image.height());
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//!
//! # Where the codecs come from
//!
//! Every codec is a separate crate that `dlopen`s its library at runtime, the way
//! [`xs-tls`](https://docs.rs/xs-tls) and `xs-egl-sys` do. Nothing is linked at
//! build time, so a device missing `libjxl.so` still builds and still decodes
//! JPEG, it just reports [`ImageError::DecoderUnavailable`] for JPEG XL. A
//! feature that is off is not compiled in at all, so its symbols never reach the
//! binary.
//!
//! | format | crate | library | notes |
//! |--------|-------|---------|-------|
//! | JPEG, JFIF, EXIF | `xs-image-jpeg` | `libjpeg.so` | lossy; grayscale stays grayscale |
//! | PNG | `xs-image-png` | `libpng16.so` | lossless; widens palettes and sub-byte samples |
//! | WebP | `xs-image-webp` | `libwebp.so` | always RGBA, straight alpha |
//! | JPEG XL | `xs-image-jpeg-xl` | `libjxl.so` | needs 0.11 or newer, output forced to sRGB |

#![warn(clippy::all, clippy::pedantic)]

mod image;
mod registry;

pub use image::{Image, ImageOptions, MAX_READ_BYTES};
pub use registry::{available_formats, build_registry, supports, with_registry};
pub use xs_image_core::{
    ColorSpace, DecoderRegistry, ImageBuffer, ImageDecoder, ImageDimensions, ImageError,
    ImageFormat, ImageResult,
};
