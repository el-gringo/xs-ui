//! Antialiased scanline rasterization.
//!
//! The outline is flattened to polylines. For each scanline, every contour's
//! crossings on that line are collected together, sorted, and walked, and the
//! run between two crossings is filled when the winding number between them is
//! not zero - the rule TrueType outlines are built for, and the reason the
//! counter of an `e` comes out empty: its contour runs the other way, so the
//! winding drops back to zero inside it.
//!
//! Filling each contour on its own and adding the results up fills that counter
//! too, which is how this started and what the `e` in the cross-check against
//! `ab_glyph` caught.

use crate::{Outline, Point};

/// How many times a pixel row is sampled top to bottom.
const SUBSAMPLES: usize = 16;

/// A glyph's outline scaled to pixels, ready to be written into an atlas.
#[derive(Clone, Debug)]
pub struct RasterizedGlyph {
    outline: Outline,
    scale: f32,
}

/// An axis aligned box in pixels, y down from the glyph's top.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Rect {
    pub min: Point,
    pub max: Point,
}

impl Rect {
    #[must_use]
    pub fn width(&self) -> f32 {
        (self.max.x - self.min.x).max(0.0)
    }

    #[must_use]
    pub fn height(&self) -> f32 {
        (self.max.y - self.min.y).max(0.0)
    }
}

impl RasterizedGlyph {
    /// Rasterizes an outline at `px_per_em`.
    ///
    /// Glyphs come from [`Glyph::rasterize`](crate::Glyph::rasterize) and
    /// [`ScaledFont::outline`](crate::ScaledFont::outline); this is here for an
    /// outline of your own, and so the fill rule can be tested without a font.
    #[must_use]
    pub fn from_outline(outline: Outline, px_per_em: f32) -> Self {
        Self {
            outline,
            scale: px_per_em,
        }
    }

    /// The glyph's rectangle in pixels, relative to the pen: x from the left
    /// side bearing, y negative above the baseline.
    ///
    /// Rounded outwards to whole pixels, so the rectangle never clips the
    /// outline and callers placing it into an atlas do not have to deal with
    /// fractional origins. An empty outline gives an empty rectangle.
    #[must_use]
    pub fn pixel_bounds(&self) -> Rect {
        let Some((min, max)) = self.outline.bounds() else {
            return Rect::default();
        };
        // Font units are y up, pixels are y down from the top of the box.
        let top_left = Point::new(min.x * self.scale, -max.y * self.scale);
        let bottom_right = Point::new(max.x * self.scale, -min.y * self.scale);
        Rect {
            min: Point::new(top_left.x.floor(), top_left.y.floor()),
            max: Point::new(bottom_right.x.ceil(), bottom_right.y.ceil()),
        }
    }

    /// The width of the rasterized glyph, in whole pixels.
    #[must_use]
    pub fn pixel_width(&self) -> u32 {
        self.pixel_bounds().width() as u32
    }

    /// The height of the rasterized glyph, in whole pixels.
    #[must_use]
    pub fn pixel_height(&self) -> u32 {
        self.pixel_bounds().height() as u32
    }

    /// The glyph's pixels, row by row, as `(x, y, coverage)` with `x` and `y`
    /// starting at zero and `y` counting down from the top of
    /// [`Self::pixel_bounds`].
    ///
    /// Every pixel of the bounding box is visited, including the fully
    /// transparent ones, so a caller can copy the rectangle without tracking
    /// which parts were touched.
    #[must_use]
    pub fn pixels(&self) -> RasterPixels {
        let bounds = self.pixel_bounds();
        let width = bounds.width() as usize;
        let height = bounds.height() as usize;
        let coverage = self.coverage(width, height, bounds);
        RasterPixels {
            width,
            height,
            coverage,
        }
    }

    /// Draws the glyph, calling `plot` once per pixel of its bounding box with
    /// coverage from `0.0` to `1.0`.
    pub fn draw(&self, mut plot: impl FnMut(u32, u32, f32)) {
        let pixels = self.pixels();
        for y in 0..pixels.height {
            for x in 0..pixels.width {
                plot(x as u32, y as u32, pixels.coverage[y * pixels.width + x]);
            }
        }
    }

    /// Coverage per pixel, `0.0` empty and `1.0` solid.
    fn coverage(&self, width: usize, height: usize, bounds: Rect) -> Vec<f32> {
        let mut coverage = vec![0.0f32; width * height];
        if width == 0 || height == 0 {
            return coverage;
        }
        // Half a pixel of tolerance: the curves have already been flattened to
        // well inside this, and it keeps a straight edge that lands exactly on a
        // pixel boundary from bleeding into its neighbour.
        let tolerance = 0.1;
        // Move every contour into the raster's own space once: origin at the
        // bounding box's top left, y down, which is how the rows are scanned.
        let to_raster = |point: Point| {
            Point::new(
                point.x * self.scale - bounds.min.x,
                -(point.y * self.scale) - bounds.min.y,
            )
        };
        let polylines: Vec<Vec<Point>> = self
            .outline
            .flatten(tolerance)
            .iter()
            .filter(|polyline| polyline.len() > 1)
            .map(|polyline| polyline.iter().map(|p| to_raster(*p)).collect())
            .collect();

        let top = polylines
            .iter()
            .flatten()
            .fold(f32::INFINITY, |acc, p| acc.min(p.y));
        let bottom = polylines
            .iter()
            .flatten()
            .fold(f32::NEG_INFINITY, |acc, p| acc.max(p.y));
        let first_row = top.floor().max(0.0) as i64;
        let last_row = (bottom.ceil() as i64).min(height as i64);

        for row in first_row..last_row {
            fill_row(&polylines, row, &mut coverage, width, height);
        }
        for value in &mut coverage {
            *value = value.min(1.0);
        }
        coverage
    }
}

/// Fills one scanline, from every contour at once.
///
/// Each crossing carries the direction it was crossed in, so the walk can tell
/// inside from outside: the winding number between two crossings is the sum of
/// the directions of everything crossed to the left of them, and a run is filled
/// when that number is not zero.
fn fill_row(polylines: &[Vec<Point>], row: i64, coverage: &mut [f32], width: usize, height: usize) {
    // Horizontal coverage is exact - a run's overlap with a pixel is measured,
    // not sampled - but a row is one sample tall, which leaves the top and
    // bottom of a stem ragged and reads as slightly thin text. So each row is
    // sampled `SUBSAMPLES` times and averaged, which is enough to make the ink
    // match a true area rasterizer to a few percent.
    for sub in 0..SUBSAMPLES {
        // Sample the middle of the sub band so it cannot land on an edge.
        let sample_y = row as f32 + (sub as f32 + 0.5) / SUBSAMPLES as f32;
        let mut crossings: Vec<(f32, i32)> = Vec::new();
        for points in polylines {
            for index in 0..points.len() {
                let from = points[index];
                let to = points[(index + 1) % points.len()];
                // Half open, so a vertex that two segments share is counted once.
                if from.y <= sample_y && to.y > sample_y {
                    let t = (sample_y - from.y) / (to.y - from.y);
                    crossings.push((from.x + t * (to.x - from.x), 1));
                } else if to.y <= sample_y && from.y > sample_y {
                    let t = (sample_y - to.y) / (from.y - to.y);
                    crossings.push((to.x + t * (from.x - to.x), -1));
                }
            }
        }
        crossings.sort_by(|a, b| a.0.total_cmp(&b.0));

        let weight = 1.0 / SUBSAMPLES as f32;
        let mut winding = 0;
        for pair in crossings.windows(2) {
            winding += pair[0].1;
            if winding != 0 {
                add_span(
                    coverage,
                    row as usize,
                    width,
                    height,
                    pair[0].0,
                    pair[1].0,
                    weight,
                );
            }
        }
    }
}

/// Adds `+/-` coverage for the horizontal run `[left, right)` on one row.
fn add_span(
    coverage: &mut [f32],
    row: usize,
    width: usize,
    height: usize,
    left: f32,
    right: f32,
    weight: f32,
) {
    if row >= height {
        return;
    }
    let (left, right) = (left.max(0.0), right.min(width as f32));
    if right <= left {
        return;
    }
    let first = left.floor().max(0.0) as usize;
    let last = (right.ceil() as usize).min(width);
    for x in first..last {
        // How much of this pixel the span covers.
        let covered = (right.min(x as f32 + 1.0) - left.max(x as f32)).max(0.0);
        if covered > 0.0 {
            coverage[row * width + x] += covered * weight;
        }
    }
}

/// The pixels of a rasterized glyph.
#[derive(Clone, Debug)]
pub struct RasterPixels {
    width: usize,
    height: usize,
    coverage: Vec<f32>,
}

impl RasterPixels {
    /// The width in pixels.
    #[must_use]
    pub fn width(&self) -> usize {
        self.width
    }

    /// The height in pixels.
    #[must_use]
    pub fn height(&self) -> usize {
        self.height
    }

    /// The coverage of one pixel, `0.0` to `1.0`.
    ///
    /// # Panics
    ///
    /// Panics when `x` or `y` is outside the rectangle.
    #[must_use]
    pub fn at(&self, x: usize, y: usize) -> f32 {
        assert!(
            x < self.width && y < self.height,
            "pixel {x},{y} out of bounds"
        );
        self.coverage[y * self.width + x]
    }

    /// The coverage of every pixel, row by row.
    #[must_use]
    pub fn coverage(&self) -> &[f32] {
        &self.coverage
    }
}

#[cfg(test)]
mod tests {
    use super::RasterizedGlyph;
    use crate::{Contour, Outline, Point, Segment};

    /// An axis aligned square as a contour, wound `clockwise` or not.
    fn square(x0: f32, y0: f32, x1: f32, y1: f32, clockwise: bool) -> Contour {
        let corners = if clockwise {
            [[x0, y0], [x1, y0], [x1, y1], [x0, y1]]
        } else {
            [[x0, y0], [x0, y1], [x1, y1], [x1, y0]]
        };
        let points: Vec<Point> = corners.iter().map(|p| Point::new(p[0], p[1])).collect();
        Contour {
            start: points[0],
            segments: points[1..].iter().map(|p| Segment::Line(*p)).collect(),
        }
    }

    /// The ink of some contours, in square pixels.
    fn ink(contours: Vec<Contour>) -> f32 {
        let outline = Outline { contours };
        RasterizedGlyph::from_outline(outline, 1.0)
            .pixels()
            .coverage()
            .iter()
            .sum()
    }

    #[test]
    fn a_square_is_fully_covered() {
        assert_eq!(ink(vec![square(0.0, 0.0, 4.0, 4.0, true)]), 16.0);
    }

    #[test]
    fn a_contour_the_other_way_round_is_still_a_hole() {
        // The fill rule is about winding, not about which way round a contour
        // happens to run, so the outer square must fill either way.
        assert_eq!(ink(vec![square(0.0, 0.0, 4.0, 4.0, false)]), 16.0);
    }

    #[test]
    fn an_opposite_wound_contour_is_a_counter() {
        // This is what makes the middle of an `e` empty, and getting it wrong
        // fills it - which is how this rasterizer got caught against `ab_glyph`.
        let ring = vec![
            square(0.0, 0.0, 4.0, 4.0, true),
            square(1.0, 1.0, 3.0, 3.0, false),
        ];
        assert_eq!(ink(ring), 12.0);
    }

    #[test]
    fn overlapping_contours_the_same_way_fill_their_union() {
        // Two 4x4 squares sharing a 2x4 column. Wound the same way the winding
        // number is 2 where they overlap, which is still inside, so the union
        // fills and nothing is punched out of it.
        let pair = vec![
            square(0.0, 0.0, 4.0, 4.0, true),
            square(2.0, 0.0, 6.0, 4.0, true),
        ];
        assert_eq!(ink(pair), 24.0);
    }

    #[test]
    fn overlapping_contours_opposite_ways_cancel_where_they_meet() {
        let pair = vec![
            square(0.0, 0.0, 4.0, 4.0, true),
            square(2.0, 0.0, 6.0, 4.0, false),
        ];
        assert_eq!(ink(pair), 16.0);
    }

    #[test]
    fn coverage_never_goes_past_solid() {
        // Same-direction overlap has winding 2, but an atlas stores 8 bit
        // coverage, so the extra winding must not leak into the pixel.
        let pair = vec![
            square(0.0, 0.0, 4.0, 4.0, true),
            square(0.0, 0.0, 4.0, 4.0, true),
        ];
        let pixels = RasterizedGlyph::from_outline(Outline { contours: pair }, 1.0).pixels();
        assert!(pixels.coverage().iter().all(|v| *v >= 0.0 && *v <= 1.0));
        assert_eq!(ink(vec![square(0.0, 0.0, 4.0, 4.0, true)]), 16.0);
    }

    #[test]
    fn an_empty_outline_is_nothing() {
        let pixels = RasterizedGlyph::from_outline(Outline::default(), 1.0).pixels();
        assert_eq!((pixels.width(), pixels.height()), (0, 0));
    }

    #[test]
    fn draw_reports_the_same_coverage_as_pixels() {
        // `draw` used to divide by 255 what was already 0.0 to 1.0, which made
        // every glyph come out almost transparent.
        let raster = RasterizedGlyph::from_outline(
            Outline {
                contours: vec![square(0.5, 0.0, 3.5, 4.0, true)],
            },
            1.0,
        );
        let pixels = raster.pixels();
        let mut drawn = Vec::new();
        raster.draw(|x, y, v| drawn.push((x, y, v)));
        assert_eq!(drawn.len(), pixels.width() * pixels.height());
        for (x, y, v) in &drawn {
            assert_eq!(*v, pixels.at(*x as usize, *y as usize), "pixel {x},{y}");
        }
        assert!(drawn.iter().any(|(_, _, v)| *v > 0.9), "nothing was drawn");
    }
}
