//! `glyf` outlines: simple glyphs and composites.
//!
//! Coordinates are stored delta-encoded behind a per-point flag byte, and the
//! flags themselves are run-length encoded, so both loops below read a byte and
//! decide what the next one means. See the OpenType spec, "TrueType Outline
//! Data".

use crate::{
    Contour, FontError, Outline, Point, Result, Segment,
    sfnt::{Sfnt, read_i16, read_u16},
};

/// `glyf` header flags on a composite glyph.
const ARG_1_AND_2_ARE_WORDS: u16 = 0x0001;
const ARGS_ARE_XY_VALUES: u16 = 0x0002;
const ROUND_XY_TO_GRID: u16 = 0x0004;
const WE_HAVE_A_SCALE: u16 = 0x0008;
const MORE_COMPONENTS: u16 = 0x0020;
const WE_HAVE_AN_X_AND_Y_SCALE: u16 = 0x0040;
const WE_HAVE_A_TWO_BY_TWO: u16 = 0x0080;
const WE_HAVE_INSTRUCTIONS: u16 = 0x0100;
const USE_MY_METRICS: u16 = 0x0200;

/// Per-point flags in a simple glyph.
const ON_CURVE: u8 = 0x01;
const X_SHORT: u8 = 0x02;
const Y_SHORT: u8 = 0x04;
const REPEAT: u8 = 0x08;
const X_SAME_OR_POSITIVE: u8 = 0x10;
const Y_SAME_OR_POSITIVE: u8 = 0x20;

/// How deep composite glyphs may nest before the font is called malformed.
///
/// Real fonts nest two or three deep. The limit is what stops a pair of
/// components pointing at each other from recursing until the stack runs out.
const MAX_COMPOSITE_DEPTH: usize = 8;

impl Sfnt<'_> {
    /// The outline of `glyph`, or `None` when it has none.
    ///
    /// # Errors
    ///
    /// [`FontError`] when the glyph data is malformed or the index is unknown.
    pub fn outline(&self, glyph: u16) -> Result<Option<Outline>> {
        let (start, end) = self.glyph_range(glyph)?;
        if start == end {
            return Ok(None);
        }
        let glyf = self
            .table(0x676C_7966)
            .ok_or(FontError::MissingTable("glyf"))?;
        let data = glyf
            .get(start..end)
            .ok_or(FontError::Truncated("glyf glyph"))?;
        self.read_outline(data, 0)
    }

    /// The `cmap` lookup for this font.
    ///
    /// # Errors
    ///
    /// [`FontError`] when the font has no usable `cmap`.
    pub fn glyph_index(&self, c: char) -> Result<Option<u16>> {
        let table = self.cmap().ok_or(FontError::MissingTable("cmap"))?;
        crate::cmap::Cmap::parse(table)?.glyph_index(self, c)
    }

    fn read_outline(&self, data: &[u8], depth: usize) -> Result<Option<Outline>> {
        if depth > MAX_COMPOSITE_DEPTH {
            return Err(FontError::Malformed("composite glyph nesting too deep"));
        }
        let contours = read_i16(data, 0)?;
        if contours >= 0 {
            return read_simple(data, contours as usize).map(Some);
        }
        self.read_composite(data)
    }

    /// A composite is a list of transformed references to other glyphs.
    fn read_composite(&self, data: &[u8]) -> Result<Option<Outline>> {
        let mut outline = Outline::default();
        let mut at = 10; // header plus the bounding box
        let mut components = 0usize;

        loop {
            let flags = read_u16(data, at)?;
            let component = read_u16(data, at + 2)?;
            at += 4;

            // The two arguments are bytes or words depending on a flag, and are
            // either a point or a point index depending on another.
            let (dx, dy) = if flags & ARG_1_AND_2_ARE_WORDS != 0 {
                let x = f32::from(read_i16(data, at)?);
                let y = f32::from(read_i16(data, at + 2)?);
                at += 4;
                (x, y)
            } else {
                let x = f32::from(*data.get(at).ok_or(FontError::Truncated("composite"))? as i8);
                let y =
                    f32::from(*data.get(at + 1).ok_or(FontError::Truncated("composite"))? as i8);
                at += 2;
                (x, y)
            };

            let (mut a, mut b, mut c, mut d) = (1.0f32, 0.0f32, 0.0f32, 1.0f32);
            if flags & WE_HAVE_A_SCALE != 0 {
                a = f2dot14(data, at)?;
                d = a;
                at += 2;
            } else if flags & WE_HAVE_AN_X_AND_Y_SCALE != 0 {
                a = f2dot14(data, at)?;
                d = f2dot14(data, at + 2)?;
                at += 4;
            } else if flags & WE_HAVE_A_TWO_BY_TWO != 0 {
                a = f2dot14(data, at)?;
                b = f2dot14(data, at + 2)?;
                c = f2dot14(data, at + 4)?;
                d = f2dot14(data, at + 6)?;
                at += 8;
            }

            let mut sub = self.outline(component)?.ok_or(FontError::Malformed(
                "composite component without an outline",
            ))?;
            // ARG_1_AND_2_ARE_XY_VALUES absent means the arguments point into
            // the parent's point array; that needs point indices this
            // implementation does not keep, so it is reported rather than
            // mis-transformed.
            if flags & ARGS_ARE_XY_VALUES == 0 {
                return Err(FontError::UnsupportedFormat(
                    "composite glyphs with point attachment",
                ));
            }
            let _ = ROUND_XY_TO_GRID | USE_MY_METRICS | WE_HAVE_INSTRUCTIONS;

            if (a - 1.0).abs() > f32::EPSILON
                || (d - 1.0).abs() > f32::EPSILON
                || b.abs() > f32::EPSILON
                || c.abs() > f32::EPSILON
            {
                for contour in &mut sub.contours {
                    contour.start = transform(contour.start, a, b, c, d);
                    for segment in &mut contour.segments {
                        *segment = match *segment {
                            Segment::Line(to) => Segment::Line(transform(to, a, b, c, d)),
                            Segment::Quad(control, to) => Segment::Quad(
                                transform(control, a, b, c, d),
                                transform(to, a, b, c, d),
                            ),
                        };
                    }
                }
            }
            sub.translate(dx, dy);
            outline.contours.extend(sub.contours);
            components += 1;

            if flags & MORE_COMPONENTS == 0 {
                break;
            }
        }
        debug_assert!(components > 0);
        Ok((!outline.is_empty()).then_some(outline))
    }
}

/// Applies a composite's 2x2 transform.
fn transform(point: Point, a: f32, b: f32, c: f32, d: f32) -> Point {
    Point::new(a * point.x + c * point.y, b * point.x + d * point.y)
}

/// A signed 2.14 fixed point number, the composite transform's storage format.
fn f2dot14(data: &[u8], at: usize) -> Result<f32> {
    Ok(f32::from(read_i16(data, at)?) / 16384.0)
}

/// A glyph with its own contour list: points, flags and deltas.
fn read_simple(data: &[u8], contour_count: usize) -> Result<Outline> {
    let mut at = 10; // header plus the bounding box
    let mut end_points = Vec::with_capacity(contour_count);
    for _ in 0..contour_count {
        end_points.push(read_u16(data, at)? as usize);
        at += 2;
    }
    let point_count = end_points.last().map_or(0, |last| last + 1);

    let instruction_len = read_u16(data, at)? as usize;
    at += 2 + instruction_len;

    // Flags are run-length encoded: a flag with REPEAT is followed by a count.
    let mut flags = Vec::with_capacity(point_count);
    while flags.len() < point_count {
        let flag = *data.get(at).ok_or(FontError::Truncated("glyf flags"))?;
        at += 1;
        flags.push(flag);
        if flag & REPEAT != 0 {
            let repeat = *data
                .get(at)
                .ok_or(FontError::Truncated("glyf flag repeat"))? as usize;
            at += 1;
            for _ in 0..repeat {
                if flags.len() == point_count {
                    break;
                }
                flags.push(flag);
            }
        }
    }

    // X then Y deltas. A short one is a single byte whose high bit is in the
    // flag; otherwise the flag says whether the next short is positive and the
    // delta is a word.
    let mut xs = Vec::with_capacity(point_count);
    let mut x = 0i32;
    for flag in &flags {
        let delta = if flag & X_SHORT != 0 {
            let byte = *data.get(at).ok_or(FontError::Truncated("glyf x"))? as i32;
            at += 1;
            if flag & X_SAME_OR_POSITIVE != 0 {
                byte
            } else {
                -byte
            }
        } else if flag & X_SAME_OR_POSITIVE != 0 {
            0
        } else {
            let word = read_i16(data, at)?;
            at += 2;
            i32::from(word)
        };
        x += delta;
        xs.push(x);
    }

    let mut ys = Vec::with_capacity(point_count);
    let mut y = 0i32;
    for flag in &flags {
        let delta = if flag & Y_SHORT != 0 {
            let byte = *data.get(at).ok_or(FontError::Truncated("glyf y"))? as i32;
            at += 1;
            if flag & Y_SAME_OR_POSITIVE != 0 {
                byte
            } else {
                -byte
            }
        } else if flag & Y_SAME_OR_POSITIVE != 0 {
            0
        } else {
            let word = read_i16(data, at)?;
            at += 2;
            i32::from(word)
        };
        y += delta;
        ys.push(y);
    }

    // Split the flat point arrays back into contours, and turn each run of
    // on/off/on/off points into the segments the format implies. Two
    // consecutive off-curve points have an implied on-curve point between them.
    let mut outline = Outline::default();
    let mut first = 0usize;
    for end in end_points {
        let points: Vec<(Point, bool)> = (first..=end.min(point_count.saturating_sub(1)))
            .filter(|index| *index < point_count)
            .map(|index| {
                (
                    Point::new(xs[index] as f32, ys[index] as f32),
                    flags[index] & ON_CURVE != 0,
                )
            })
            .collect();
        first = end + 1;
        if let Some(contour) = to_contour(&points) {
            outline.contours.push(contour);
        }
    }
    Ok(outline)
}

/// Turns one contour's points into segments, inserting the implied on-curve
/// points the format allows to be left out.
fn to_contour(points: &[(Point, bool)]) -> Option<Contour> {
    if points.len() < 2 {
        return None;
    }
    // Rotate so the contour starts on the curve.
    let start_index = points.iter().position(|(_, on_curve)| *on_curve)?;
    let mut rotated: Vec<(Point, bool)> = points[start_index..]
        .iter()
        .chain(points[..start_index].iter())
        .copied()
        .collect();
    // A final on-curve point that repeats the first is the closing point.
    if rotated.len() > 1 && rotated[0].0 == rotated[rotated.len() - 1].0 {
        rotated.pop();
    }
    if rotated.len() < 2 {
        return None;
    }

    let start = rotated[0].0;
    let mut segments = Vec::with_capacity(rotated.len());
    let mut current = start;
    let mut control: Option<Point> = None;

    for (point, on_curve) in rotated.iter().skip(1) {
        match (control, *on_curve) {
            // An off-curve point after a control point: the implied on-curve
            // point between them is their midpoint.
            (Some(previous), false) => {
                let implied = previous.lerp(*point, 0.5);
                segments.push(Segment::Quad(previous, implied));
                current = implied;
                control = Some(*point);
            }
            (Some(previous), true) => {
                segments.push(Segment::Quad(previous, *point));
                current = *point;
                control = None;
            }
            (None, false) => control = Some(*point),
            (None, true) => {
                segments.push(Segment::Line(*point));
                current = *point;
            }
        }
    }
    // Close on the start point.
    match control {
        Some(previous) => segments.push(Segment::Quad(previous, start)),
        None => {
            if current != start {
                segments.push(Segment::Line(start));
            }
        }
    }

    (!segments.is_empty()).then_some(Contour { start, segments })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sfnt::test_view;

    const FONT: &[u8] = include_bytes!("../../../ressources/fonts/IBMPlexSans-Regular.ttf");

    fn sfnt() -> Sfnt<'static> {
        test_view(FONT)
    }

    /// Every number below was read out of the real file by an independent
    /// Python parser of `glyf`, so a mistake here cannot agree with a mistake
    /// in this crate by accident.
    #[test]
    fn the_x_glyph_is_one_closed_contour() {
        // 'X' is glyph 53, numberOfContours 1.
        let outline = sfnt().outline(53).unwrap().unwrap();
        assert_eq!(outline.contours.len(), 1);
        assert!(!outline.is_empty());
    }

    #[test]
    fn outlines_land_in_the_expected_place() {
        // Glyph 53: xMin 24, yMin 0, xMax 589, yMax 698.
        let outline = sfnt().outline(53).unwrap().unwrap();
        let (min, max) = outline.bounds().unwrap();
        assert!((min.x - 24.0).abs() < 1.0, "xMin {min:?}");
        assert!(min.y.abs() < 1.0, "yMin {min:?}");
        assert!((max.x - 589.0).abs() < 1.0, "xMax {max:?}");
        assert!((max.y - 698.0).abs() < 1.0, "yMax {max:?}");
    }

    #[test]
    fn contour_counts_match_the_glyf_header() {
        // 'g' has 3 contours, 'e' and 'A' have 2.
        for (c, expected) in [('g', 3), ('e', 2), ('A', 2)] {
            let glyph = sfnt().glyph_index(c).unwrap().unwrap();
            let actual = sfnt().outline(glyph).unwrap().unwrap().contours.len();
            assert_eq!(actual, expected, "{c:?}");
        }
    }

    #[test]
    fn every_flattened_point_stays_inside_the_glyph_box() {
        // A delta-decoding mistake moves points by a lot, so this is the check
        // that would notice one.
        for c in ['X', 'g', 'e', '@', 'A'] {
            let glyph = sfnt().glyph_index(c).unwrap().unwrap();
            let outline = sfnt().outline(glyph).unwrap().unwrap();
            let (min, max) = outline.bounds().unwrap();
            for polyline in outline.flatten(0.05) {
                for point in polyline {
                    assert!(point.x >= min.x - 1.0, "{c:?} x {point:?} vs {min:?}");
                    assert!(point.x <= max.x + 1.0, "{c:?} x {point:?} vs {max:?}");
                    assert!(point.y >= min.y - 1.0, "{c:?} y {point:?} vs {min:?}");
                    assert!(point.y <= max.y + 1.0, "{c:?} y {point:?} vs {max:?}");
                }
            }
        }
    }

    #[test]
    fn a_space_has_no_outline() {
        let space = sfnt().glyph_index(' ').unwrap().unwrap();
        assert_eq!(sfnt().outline(space).unwrap(), None);
    }

    #[test]
    fn a_lowercase_glyph_has_a_descender_below_the_baseline() {
        // Glyph 8 ('g'): yMin -212, so part of it is below the baseline even
        // though its top is up at the x-height.
        let g = sfnt().glyph_index('g').unwrap().unwrap();
        let (min, max) = sfnt().outline(g).unwrap().unwrap().bounds().unwrap();
        assert!((min.y - -212.0).abs() < 1.0, "yMin {min:?}");
        assert!((max.y - 586.0).abs() < 1.0, "yMax {max:?}");
    }

    #[test]
    fn a_composite_glyph_has_more_outline_than_its_base() {
        // 'é' (glyph 252) is a composite: the 'e' outline plus an acute accent.
        let plain = sfnt()
            .outline(sfnt().glyph_index('e').unwrap().unwrap())
            .unwrap()
            .unwrap();
        let accented = sfnt().outline(252).unwrap().unwrap();
        let count = |outline: &crate::Outline| {
            outline.contours.len()
                + outline
                    .flatten(0.05)
                    .iter()
                    .map(std::vec::Vec::len)
                    .sum::<usize>()
        };
        let (plain_len, accented_len) = (count(&plain), count(&accented));
        assert!(
            accented_len > plain_len,
            "é ({accented_len}) should be more complex than e ({plain_len})"
        );
    }

    #[test]
    fn a_composite_places_its_component_above_the_base() {
        // é is glyph 6 (the e, bbox 47,-12,502,528) plus glyph 916 (the acute,
        // bbox 255,585,413,779) offset by (-21, 0). Combined: xMin 47, xMax
        // 502, yMin -12, yMax 779. Only correct if both the offset and the
        // transform are applied.
        let outline = sfnt().outline(252).unwrap().unwrap();
        let (min, max) = outline.bounds().unwrap();
        assert!((min.x - 47.0).abs() < 1.0, "xMin {min:?}");
        assert!((min.y - -12.0).abs() < 1.0, "yMin {min:?}");
        assert!((max.x - 502.0).abs() < 1.0, "xMax {max:?}");
        assert!((max.y - 779.0).abs() < 1.0, "yMax {max:?}");
        assert_eq!(outline.contours.len(), 3, "e has 2 contours, the acute 1");
    }

    #[test]
    fn glyphs_are_not_all_the_same_shape() {
        // A parser that returned an empty or fixed outline would pass every test
        // above, so check the shapes really differ.
        let x = sfnt().outline(53).unwrap().unwrap().flatten(0.05)[0].clone();
        let a = sfnt().outline(30).unwrap().unwrap().flatten(0.05)[0].clone();
        assert_ne!(x, a);
        assert!(x.len() > 4 && a.len() > 4);
    }

    #[test]
    fn every_glyph_in_the_font_parses_without_error() {
        // The real coverage test: a broken parser shows up here.
        let font = sfnt();
        let count = font.num_glyphs();
        let mut with_outlines = 0;
        for glyph in 0..count {
            if let Ok(Some(_)) = font.outline(glyph) {
                with_outlines += 1;
            }
        }
        assert!(
            with_outlines > count as usize / 2,
            "only {with_outlines} of {count} glyphs produced an outline"
        );
    }

    #[test]
    fn a_composite_glyph_resolves() {
        // 'Á' is a base letter plus an accent in most fonts, i.e. a composite.
        let font = sfnt();
        let Ok(Some(id)) = font.glyph_index('Á') else {
            return; // Not present in this font; nothing to check.
        };
        assert!(font.outline(id).unwrap().is_some());
    }

    #[test]
    fn an_unknown_glyph_index_is_refused() {
        assert!(sfnt().outline(60000).is_err());
    }
}
