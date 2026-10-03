//! End to end tests for the facade: every fixture goes in through `Image::read`,
//! which is the path an application takes.

use xs_image::{
    ColorSpace, Image, ImageError, ImageFormat, ImageOptions, available_formats, supports,
};

const RGB_JPEG: &[u8] = include_bytes!("../testdata/rgb.jpg");
const GRAYSCALE_JPEG: &[u8] = include_bytes!("../testdata/grayscale.jpg");
const RGB_PNG: &[u8] = include_bytes!("../testdata/rgb.png");
const RGBA_PNG: &[u8] = include_bytes!("../testdata/rgba.png");
const PALETTE_PNG: &[u8] = include_bytes!("../testdata/palette.png");
const INTERLACED_PNG: &[u8] = include_bytes!("../testdata/interlaced.png");
const RGBA_WEBP: &[u8] = include_bytes!("../testdata/rgba.webp");
const RGBA_JXL: &[u8] = include_bytes!("../testdata/rgba.jxl");

/// The four colours baked into every fixture.
const RGB: [u8; 12] = [255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
const RGBA: [u8; 16] = [
    255, 0, 0, 255, 0, 255, 0, 128, 0, 0, 255, 255, 255, 255, 255, 1,
];

// --------------------------------------------------
// Dispatch
// --------------------------------------------------
#[test]
fn sniffs_every_fixture_to_its_own_format() {
    for (bytes, expected) in [
        (RGB_JPEG, ImageFormat::Jpeg),
        (GRAYSCALE_JPEG, ImageFormat::Jpeg),
        (RGB_PNG, ImageFormat::Png),
        (RGBA_PNG, ImageFormat::Png),
        (RGBA_WEBP, ImageFormat::WebP),
        (RGBA_JXL, ImageFormat::JpegXl),
    ] {
        let image = Image::read(bytes, ImageOptions::default()).unwrap();
        assert_eq!(image.format(), Some(expected), "{}", expected.name());
    }
}

#[test]
fn every_declared_codec_is_reachable() {
    let formats = available_formats();
    for format in ImageFormat::ALL {
        assert!(
            supports(format),
            "no decoder for {} on this device",
            format.name()
        );
    }
    assert_eq!(formats.len(), ImageFormat::ALL.len());
}

#[test]
fn each_format_decodes_to_two_by_two() {
    for (bytes, name) in [
        (RGB_JPEG, "rgb.jpg"),
        (RGB_PNG, "rgb.png"),
        (RGBA_WEBP, "rgba.webp"),
        (RGBA_JXL, "rgba.jxl"),
    ] {
        let image = Image::read(bytes, ImageOptions::default()).unwrap();
        assert_eq!(image.dimensions(), (2, 2), "{name}");
    }
}

// --------------------------------------------------
// Samples
// --------------------------------------------------
#[test]
fn a_lossless_rgb_png_gives_the_exact_pixels() {
    let image = Image::read(RGB_PNG, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgb);
    test_utils::assert_slice_eq(&RGB, image.samples());
}

#[test]
fn a_lossless_rgba_png_gives_the_exact_pixels() {
    let image = Image::read(RGBA_PNG, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgba);
    test_utils::assert_slice_eq(&RGBA, image.samples());
}

#[test]
fn a_lossless_webp_gives_the_exact_pixels() {
    let image = Image::read(RGBA_WEBP, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgba);
    test_utils::assert_slice_eq(&RGBA, image.samples());
}

#[test]
fn a_lossless_jpeg_xl_gives_the_exact_pixels() {
    let image = Image::read(RGBA_JXL, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgba);
    test_utils::assert_slice_eq(&RGBA, image.samples());
}

#[test]
fn a_palette_png_is_widened_to_rgba() {
    let image = Image::read(PALETTE_PNG, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgba);
    test_utils::assert_slice_eq(&RGBA, image.samples());
}

#[test]
fn an_interlaced_png_is_de_interlaced() {
    let image = Image::read(INTERLACED_PNG, ImageOptions::default()).unwrap();
    assert_eq!(image.dimensions(), (8, 8));
    let mut expected = Vec::new();
    for y in 0..8u8 {
        for x in 0..8u8 {
            expected.extend_from_slice(&[x * 32, y * 32, 128]);
        }
    }
    test_utils::assert_slice_eq(&expected, image.samples());
}

#[test]
fn a_lossy_jpeg_is_within_a_couple_of_steps() {
    let image = Image::read(RGB_JPEG, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgb);
    for (index, (actual, expected)) in image.samples().iter().zip(RGB.iter()).enumerate() {
        assert!(
            i32::from(*actual) - i32::from(*expected).abs() <= 2,
            "sample {index} is {actual}, expected {expected} +/- 2"
        );
    }
}

// --------------------------------------------------
// Colour space
// --------------------------------------------------
#[test]
fn grayscale_jpeg_stays_grayscale_until_asked() {
    let image = Image::read(GRAYSCALE_JPEG, ImageOptions::default()).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Luma);
    assert_eq!(image.samples().len(), 4);
}

#[test]
fn convert_color_widens_grayscale_to_rgba() {
    let mut image = Image::read(GRAYSCALE_JPEG, ImageOptions::default()).unwrap();
    image.convert_color(ColorSpace::Rgba).unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgba);
    assert_eq!(image.samples().len(), 4 * 4);
    // 0, 64, 128, 255 as opaque grey.
    test_utils::assert_slice_eq(&[0, 0, 0, 255, 64, 64, 64, 255], &image.samples()[0..8]);
}

#[test]
fn the_color_space_option_widens_during_read() {
    let image = Image::read(
        GRAYSCALE_JPEG,
        ImageOptions::default().color_space(ColorSpace::Rgba),
    )
    .unwrap();
    assert_eq!(image.color_space(), ColorSpace::Rgba);
    assert_eq!(image.samples().len(), 4 * 4);
}

#[test]
fn convert_color_to_an_unsupported_space_is_refused() {
    let mut image = Image::read(RGB_PNG, ImageOptions::default()).unwrap();
    let actual = image.convert_color(ColorSpace::Luma).unwrap_err();
    assert!(matches!(
        actual,
        ImageError::UnsupportedConversion(ColorSpace::Rgb, ColorSpace::Luma)
    ));
}

#[test]
fn convert_color_to_the_current_space_is_a_no_op() {
    let mut image = Image::read(RGB_PNG, ImageOptions::default()).unwrap();
    let before = image.samples().to_vec();
    image.convert_color(ColorSpace::Rgb).unwrap();
    test_utils::assert_slice_eq(&before, image.samples());
}

// --------------------------------------------------
// Header only
// --------------------------------------------------
#[test]
fn read_dimensions_matches_the_decoded_size_for_every_fixture() {
    for (bytes, name) in [
        (RGB_JPEG, "rgb.jpg"),
        (GRAYSCALE_JPEG, "grayscale.jpg"),
        (RGBA_PNG, "rgba.png"),
        (INTERLACED_PNG, "interlaced.png"),
        (RGBA_WEBP, "rgba.webp"),
        (RGBA_JXL, "rgba.jxl"),
    ] {
        let dimensions = Image::read_dimensions(bytes).unwrap();
        let image = Image::read(bytes, ImageOptions::default()).unwrap();
        assert_eq!(dimensions.as_tuple(), image.dimensions(), "{name}");
    }
}

// --------------------------------------------------
// Failures
// --------------------------------------------------
#[test]
fn bytes_no_format_claims_are_rejected() {
    let actual = Image::read(b"this is not an image", ImageOptions::default()).unwrap_err();
    assert!(matches!(actual, ImageError::UnknownFormat));
}

#[test]
fn a_truncated_file_is_rejected_without_killing_the_process() {
    // Every codec here calls exit() or abort() by default; the setjmp handlers are
    // what keep these from taking the test runner down.
    for (bytes, name) in [
        (RGB_JPEG, "rgb.jpg"),
        (RGBA_PNG, "rgba.png"),
        (RGBA_WEBP, "rgba.webp"),
        (RGBA_JXL, "rgba.jxl"),
    ] {
        let truncated = &bytes[..bytes.len() / 2];
        assert!(
            Image::read(truncated, ImageOptions::default()).is_err(),
            "{name} truncated should not decode"
        );
    }
}

#[test]
fn a_failure_does_not_poison_the_next_decode() {
    let _ = Image::read(b"garbage", ImageOptions::default());
    let _ = Image::read(&RGB_JPEG[..3], ImageOptions::default());
    let _ = Image::read(&RGBA_PNG[..10], ImageOptions::default());
    assert!(Image::read(RGB_PNG, ImageOptions::default()).is_ok());
}

// --------------------------------------------------
// The renderer contract
// --------------------------------------------------
#[test]
fn asking_for_rgba_always_yields_four_channels_per_pixel() {
    // `xs-backend-wayland` hands these samples to `glTexImage2D` as `GL_RGBA`, and
    // `Application::read_messages` sizes the texture from width and height alone.
    // A short buffer there is a garbage texture, not an error, so every format has
    // to land on exactly `width * height * 4`.
    for (bytes, name) in [
        (RGB_JPEG, "rgb.jpg"),
        (GRAYSCALE_JPEG, "grayscale.jpg"),
        (RGB_PNG, "rgb.png"),
        (RGBA_PNG, "rgba.png"),
        (PALETTE_PNG, "palette.png"),
        (INTERLACED_PNG, "interlaced.png"),
        (RGBA_WEBP, "rgba.webp"),
        (RGBA_JXL, "rgba.jxl"),
    ] {
        let image =
            Image::read(bytes, ImageOptions::default().color_space(ColorSpace::Rgba)).unwrap();
        let (width, height) = image.dimensions();
        let expected = width as usize * height as usize * 4;
        assert_eq!(image.samples().len(), expected, "{name}");
    }
}

// --------------------------------------------------
// Streaming
// --------------------------------------------------
#[test]
fn read_from_a_reader_matches_read_from_a_slice() {
    let mut cursor = std::io::Cursor::new(RGBA_PNG.to_vec());
    let streamed = Image::read_from(&mut cursor, ImageOptions::default()).unwrap();
    let direct = Image::read(RGBA_PNG, ImageOptions::default()).unwrap();
    test_utils::assert_slice_eq(direct.samples(), streamed.samples());
}

#[test]
fn read_from_reports_a_failing_reader() {
    struct Failing;
    impl std::io::Read for Failing {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("no"))
        }
    }
    let actual = Image::read_from(&mut Failing, ImageOptions::default()).unwrap_err();
    assert!(matches!(actual, ImageError::Io(_)));
}
