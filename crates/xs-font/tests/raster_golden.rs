//! Golden rasterization values, so a change in the rasterizer has to be a
//! deliberate one.
//!
//! These were captured from this crate only after its output was checked against
//! `ab_glyph`, the crate it replaced, over all 94 printable ASCII glyphs at
//! several sizes: the rectangles matched exactly and total ink agreed to under
//! 1% on average. `ab_glyph` is gone, so these stand in for it.
//!
//! Sizes are line heights rather than ems, for the reason
//! [`Font::px_per_em_for_line_height`] explains.

const FONT: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

/// `(char, glyph index, line height px, ink in px^2, x, y, width, height)`
///
/// The rectangle is relative to the pen, with y negative above the baseline,
/// which is how an atlas places a glyph.
type Golden = (char, u16, f32, f32, i32, i32, i32, i32);

const GOLDEN: &[Golden] = &[
    ('X', 53, 16.0, 19.398, 0, -9, 8, 9),
    ('A', 30, 16.0, 19.646, 0, -9, 8, 9),
    ('e', 6, 16.0, 16.560, 0, -7, 7, 8),
    ('o', 18, 16.0, 15.383, 0, -7, 7, 8),
    ('8', 66, 16.0, 23.413, 0, -9, 7, 10),
    ('@', 81, 16.0, 34.412, 0, -9, 11, 11),
    ('g', 8, 16.0, 23.834, 0, -8, 7, 11),
    ('W', 52, 16.0, 30.738, 0, -9, 11, 9),
    ('é', 252, 16.0, 18.545, 0, -10, 7, 11),
    ('B', 31, 16.0, 26.377, 1, -9, 7, 9),
    ('k', 14, 16.0, 17.175, 1, -10, 6, 10),
    ('S', 48, 16.0, 19.000, 0, -9, 7, 10),
    ('?', 111, 16.0, 12.546, 0, -9, 6, 10),
    ('i', 12, 16.0, 7.561, 0, -10, 3, 10),
    ('(', 112, 16.0, 9.958, 1, -10, 4, 12),
    ('X', 53, 32.0, 77.897, 0, -18, 15, 18),
    ('A', 30, 32.0, 79.185, 0, -18, 16, 18),
    ('e', 6, 32.0, 65.788, 1, -13, 12, 14),
    ('o', 18, 32.0, 61.534, 1, -13, 12, 14),
    ('8', 66, 32.0, 93.646, 1, -18, 13, 19),
    ('@', 81, 32.0, 137.763, 1, -18, 20, 21),
    ('g', 8, 32.0, 95.588, 1, -15, 12, 21),
    ('W', 52, 32.0, 123.371, 0, -18, 22, 18),
    ('é', 252, 32.0, 73.724, 1, -20, 12, 21),
    ('B', 31, 32.0, 105.641, 2, -18, 13, 18),
    ('k', 14, 32.0, 68.434, 2, -19, 11, 19),
    ('S', 48, 32.0, 76.009, 1, -18, 13, 19),
    ('?', 111, 32.0, 50.179, 0, -18, 11, 19),
    ('i', 12, 32.0, 30.111, 1, -19, 4, 19),
    ('(', 112, 32.0, 39.716, 2, -19, 7, 23),
    ('X', 53, 64.0, 311.586, 1, -35, 28, 35),
    ('A', 30, 64.0, 317.480, 1, -35, 30, 35),
    ('e', 6, 64.0, 265.136, 2, -26, 23, 27),
    ('o', 18, 64.0, 246.108, 2, -26, 24, 27),
    ('8', 66, 64.0, 374.624, 2, -35, 25, 36),
    ('@', 81, 64.0, 551.007, 3, -35, 38, 41),
    ('g', 8, 64.0, 382.124, 2, -29, 24, 40),
    ('W', 52, 64.0, 493.484, 0, -35, 43, 35),
    ('é', 252, 64.0, 296.882, 2, -39, 23, 40),
    ('B', 31, 64.0, 423.212, 4, -35, 26, 35),
    ('k', 14, 64.0, 273.970, 4, -37, 22, 37),
    ('S', 48, 64.0, 303.986, 2, -35, 25, 36),
    ('?', 111, 64.0, 200.684, 1, -35, 21, 36),
    ('i', 12, 64.0, 120.486, 3, -37, 6, 37),
    ('(', 112, 64.0, 159.098, 4, -38, 13, 45),
];

#[test]
fn the_rectangles_and_ink_are_unchanged() {
    let font = xs_font::Font::parse(FONT.to_vec()).unwrap();
    for &(c, id, size, ink, x, y, w, h) in GOLDEN {
        let scaled = font.scaled(font.px_per_em_for_line_height(size));
        assert_eq!(scaled.glyph_index(c), Some(id), "{c:?} glyph index");

        let raster = scaled
            .outline(id)
            .unwrap_or_else(|| panic!("{c:?} has no outline"));
        let bounds = raster.pixel_bounds();
        assert_eq!(
            (
                bounds.min.x as i32,
                bounds.min.y as i32,
                bounds.width() as i32,
                bounds.height() as i32
            ),
            (x, y, w, h),
            "{c:?} at {size}px rectangle"
        );

        let pixels = raster.pixels();
        assert_eq!(
            (pixels.width(), pixels.height()),
            (w as usize, h as usize),
            "{c:?} at {size}px raster size"
        );
        let total: f32 = pixels.coverage().iter().sum();
        // Coverage is a float sum over a scanline rasterizer, so this is a
        // tolerance rather than an equality.
        assert!(
            (total - ink).abs() < 0.01,
            "{c:?} at {size}px ink {total} vs {ink}"
        );
    }
}

#[test]
fn a_glyph_with_no_outline_has_no_pixels() {
    let font = xs_font::Font::parse(FONT.to_vec()).unwrap();
    let scaled = font.scaled(32.0);
    let space = scaled.glyph_index(' ').unwrap();
    assert!(
        scaled.outline(space).is_none(),
        "a space should not rasterize"
    );
}
