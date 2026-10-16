use crate::{Rectangle, Vector2};
use std::ops::Mul;

/// A 3x3 matrix for 2D affine transforms.
///
/// Stored **column-major**, which is the layout GLSL's `mat3` uses, so
/// [`Matrix3::to_array`] can be handed straight to `glUniformMatrix3fv` with
/// `transpose` false. `columns[c][r]` is therefore row `r` of column `c`.
///
/// This crate's screen space has y growing **downwards** (top-left origin), so a
/// positive [`Matrix3::rotation`] reads as clockwise on screen.
#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Matrix3 {
    /// The three columns, each three rows tall.
    pub columns: [[f32; 3]; 3],
}

impl Matrix3 {
    /// The identity transform: leaves every point where it is.
    pub fn identity() -> Matrix3 {
        Matrix3 {
            columns: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// A pure translation by `offset`.
    pub fn translation(offset: Vector2) -> Matrix3 {
        Matrix3 {
            columns: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [offset.x, offset.y, 1.0]],
        }
    }

    /// A pure scale, per axis.
    pub fn scale(factor: Vector2) -> Matrix3 {
        Matrix3 {
            columns: [[factor.x, 0.0, 0.0], [0.0, factor.y, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// A pure scale by the same amount on both axes.
    pub fn uniform_scale(factor: f32) -> Matrix3 {
        Matrix3::scale(Vector2::new(factor, factor))
    }

    /// A pure rotation of `radians`, about the origin.
    ///
    /// In this crate's y-down screen space a positive angle turns clockwise.
    pub fn rotation(radians: f32) -> Matrix3 {
        let (sin, cos) = radians.sin_cos();
        Matrix3 {
            columns: [[cos, sin, 0.0], [-sin, cos, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    /// `self` followed by `other`.
    ///
    /// This is the ordinary matrix product `self * other`, so it reads as the
    /// order the transforms are *applied*: `a * b` moves a point with `b` first
    /// and then with `a`. That matches composing onto the left, which is what
    /// [`Mul`] gives you and what GLSL's `u_transform * vec3(position, 1.0)` in
    /// the shaders expects.
    pub fn then(&self, other: &Matrix3) -> Matrix3 {
        *self * *other
    }

    /// Moves `point` by this transform, translation included.
    pub fn transform_point(&self, point: Vector2) -> Vector2 {
        let x = self.columns[0][0] * point.x + self.columns[1][0] * point.y + self.columns[2][0];
        let y = self.columns[0][1] * point.x + self.columns[1][1] * point.y + self.columns[2][1];
        Vector2::new(x, y)
    }

    /// Moves `vector` by this transform's linear part, ignoring translation.
    pub fn transform_vector(&self, vector: Vector2) -> Vector2 {
        let x = self.columns[0][0] * vector.x + self.columns[1][0] * vector.y;
        let y = self.columns[0][1] * vector.x + self.columns[1][1] * vector.y;
        Vector2::new(x, y)
    }

    /// The axis-aligned bounding box of `rect` after this transform.
    ///
    /// Under a rotation the transformed rectangle is no longer axis-aligned, so
    /// this is the box around its four transformed corners rather than the
    /// rectangle itself. That is the right answer for a clip region, which is
    /// what the render driver needs, and a conservative one: it can clip less
    /// than the exact shape, never more.
    pub fn transform_rect_bounds(&self, rect: Rectangle) -> Rectangle {
        let corners = [
            self.transform_point(Vector2::new(rect.x, rect.y)),
            self.transform_point(Vector2::new(rect.x + rect.width, rect.y)),
            self.transform_point(Vector2::new(rect.x + rect.width, rect.y + rect.height)),
            self.transform_point(Vector2::new(rect.x, rect.y + rect.height)),
        ];
        let min_x = corners.iter().fold(f32::INFINITY, |m, c| m.min(c.x));
        let min_y = corners.iter().fold(f32::INFINITY, |m, c| m.min(c.y));
        let max_x = corners.iter().fold(f32::NEG_INFINITY, |m, c| m.max(c.x));
        let max_y = corners.iter().fold(f32::NEG_INFINITY, |m, c| m.max(c.y));
        Rectangle::new(min_x, min_y, max_x - min_x, max_y - min_y)
    }

    /// The uniform scale this transform applies, as the geometric mean of its two
    /// axis scales.
    ///
    /// Exact for a rigid transform (returns `1.0`) and for a uniform scale. Under a
    /// non-uniform scale there is no single right answer, and this returns the
    /// compromise: a shape squashed to half height reports `sqrt(0.5)`, which
    /// describes its area rather than either axis.
    ///
    /// This is what a shader that resolves coverage in the element's own pixels
    /// needs, because the distance from a sample to a curve has to be expressed in
    /// the pixels that are actually being drawn. Getting it wrong does not look
    /// wrong at a glance: the glyph still appears, but pixels outside the ink pick
    /// up a little coverage, which reads as a grey slab behind each glyph.
    pub fn scale_factor(&self) -> f32 {
        let (a, b) = (self.columns[0][0], self.columns[1][0]);
        let (c, d) = (self.columns[0][1], self.columns[1][1]);
        (a * d - b * c).abs().sqrt()
    }

    /// Scales the translation by `scale`, leaving the linear part alone.
    ///
    /// This is what converting a style authored in logical pixels into device
    /// pixels needs: a `translate(10, 0)` is ten pixels however the element is
    /// rotated, so the offset has to be scaled after it has been composed
    /// rather than before. Scaling the column in place is exactly that, because
    /// the column already holds the composed offset.
    ///
    /// Scaling the linear part here would be wrong - `scale(2)` is a factor,
    /// not a length, and doubling it changes what the element looks like
    /// instead of how many device pixels it covers.
    #[must_use]
    pub fn scale_translation(mut self, scale: f32) -> Matrix3 {
        self.columns[2][0] *= scale;
        self.columns[2][1] *= scale;
        self
    }

    /// The nine elements in column-major order, ready for `u_transform`.
    pub fn to_array(&self) -> [f32; 9] {
        let c = &self.columns;
        [
            c[0][0], c[0][1], c[0][2], c[1][0], c[1][1], c[1][2], c[2][0], c[2][1], c[2][2],
        ]
    }
}

impl Default for Matrix3 {
    /// The identity, so a `Matrix3` starts out doing nothing rather than
    /// collapsing everything to a point.
    fn default() -> Matrix3 {
        Matrix3::identity()
    }
}

impl Mul for Matrix3 {
    type Output = Matrix3;

    fn mul(self, rhs: Matrix3) -> Matrix3 {
        let mut columns = [[0.0f32; 3]; 3];
        for (c, column) in columns.iter_mut().enumerate() {
            for (r, cell) in column.iter_mut().enumerate() {
                // rhs column c dotted with self row r.
                *cell = rhs.columns[c][0] * self.columns[0][r]
                    + rhs.columns[c][1] * self.columns[1][r]
                    + rhs.columns[c][2] * self.columns[2][r];
            }
        }
        Matrix3 { columns }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-5
    }

    fn assert_point(m: &Matrix3, point: Vector2, expected: Vector2) {
        let actual = m.transform_point(point);
        assert!(
            close(actual.x, expected.x) && close(actual.y, expected.y),
            "{m:?} moved {point:?} to {actual:?}, expected {expected:?}"
        );
    }

    #[test]
    fn identity_changes_nothing() {
        let m = Matrix3::identity();
        assert_point(&m, Vector2::new(3.0, -7.0), Vector2::new(3.0, -7.0));
        assert_eq!(m.to_array()[8], 1.0);
    }

    #[test]
    fn default_is_the_identity() {
        assert_eq!(Matrix3::default(), Matrix3::identity());
    }

    #[test]
    fn translation_only_moves_the_point() {
        let m = Matrix3::translation(Vector2::new(10.0, 20.0));
        assert_point(&m, Vector2::new(1.0, 2.0), Vector2::new(11.0, 22.0));
        // A vector ignores the translation.
        assert_eq!(
            m.transform_vector(Vector2::new(1.0, 2.0)),
            Vector2::new(1.0, 2.0)
        );
    }

    #[test]
    fn scale_multiplies_each_axis() {
        let m = Matrix3::scale(Vector2::new(2.0, 3.0));
        assert_point(&m, Vector2::new(4.0, 5.0), Vector2::new(8.0, 15.0));
    }

    #[test]
    fn uniform_scale_is_scale_on_both_axes() {
        assert_eq!(
            Matrix3::uniform_scale(2.0),
            Matrix3::scale(Vector2::new(2.0, 2.0))
        );
    }

    #[test]
    fn rotation_by_a_quarter_turn_is_the_axis_swap() {
        let m = Matrix3::rotation(std::f32::consts::FRAC_PI_2);
        // y points down, so +90 degrees is the (x, y) -> (-y, x) rotation.
        assert_point(&m, Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0));
        assert_point(&m, Vector2::new(0.0, 1.0), Vector2::new(-1.0, 0.0));
    }

    #[test]
    fn rotation_preserves_length() {
        let m = Matrix3::rotation(0.7);
        let before = Vector2::new(3.0, 4.0);
        let after = m.transform_vector(before);
        assert!(close(before.magnitude(), after.magnitude()));
    }

    #[test]
    fn multiplying_applies_the_right_hand_side_first() {
        // Scale by 2, then translate by 10: a point at 1 lands at 12, not 22.
        let m = Matrix3::translation(Vector2::new(10.0, 0.0)) * Matrix3::uniform_scale(2.0);
        assert_point(&m, Vector2::new(1.0, 0.0), Vector2::new(12.0, 0.0));

        // The other way round is translate by 20.
        let m = Matrix3::uniform_scale(2.0) * Matrix3::translation(Vector2::new(10.0, 0.0));
        assert_point(&m, Vector2::new(1.0, 0.0), Vector2::new(22.0, 0.0));
    }

    #[test]
    fn then_matches_the_multiplication_operator() {
        let a = Matrix3::rotation(0.3);
        let b = Matrix3::translation(Vector2::new(5.0, 6.0));
        assert_eq!(a.then(&b), a * b);
    }

    #[test]
    fn composing_with_the_identity_is_a_no_op() {
        let m = Matrix3::rotation(0.4) * Matrix3::uniform_scale(1.5);
        assert_eq!(m * Matrix3::identity(), m);
        assert_eq!(Matrix3::identity() * m, m);
    }

    #[test]
    fn array_is_column_major() {
        // The translation is the third column, so it is elements 6 and 7.
        let array = Matrix3::translation(Vector2::new(30.0, 40.0)).to_array();
        assert_eq!(array, [1., 0., 0., 0., 1., 0., 30., 40., 1.]);
    }

    #[test]
    fn an_identity_leaves_a_rectangle_alone() {
        let r = Rectangle::new(10.0, 20.0, 30.0, 40.0);
        let actual = Matrix3::identity().transform_rect_bounds(r);
        assert_eq!((actual.x, actual.y), (10.0, 20.0));
        assert_eq!((actual.width, actual.height), (30.0, 40.0));
    }

    #[test]
    fn a_translation_moves_the_bounds_whole() {
        let r = Rectangle::new(10.0, 20.0, 30.0, 40.0);
        let actual = Matrix3::translation(Vector2::new(5.0, -5.0)).transform_rect_bounds(r);
        assert_eq!((actual.x, actual.y), (15.0, 15.0));
        assert_eq!((actual.width, actual.height), (30.0, 40.0));
    }

    #[test]
    fn a_quarter_turn_grows_the_bounds() {
        // 30x40 turned through 90 degrees is a 40x30 box.
        let r = Rectangle::new(0.0, 0.0, 30.0, 40.0);
        let actual = Matrix3::rotation(std::f32::consts::FRAC_PI_2).transform_rect_bounds(r);
        assert!(close(actual.width, 40.0), "width was {}", actual.width);
        assert!(close(actual.height, 30.0), "height was {}", actual.height);
    }

    #[test]
    fn rotating_about_a_corner_keeps_that_corner_put() {
        // A rotation about `pivot` is `T(pivot) * R * T(-pivot)`: the right-hand
        // side runs first, so the point is moved to the origin, turned, and put
        // back. Leaving off either `T` gives a different transform entirely -
        // `R * T(-pivot)` sends the pivot to the origin.
        let pivot = Vector2::new(10.0, 20.0);
        let m = Matrix3::translation(pivot)
            * Matrix3::rotation(0.5)
            * Matrix3::translation(Vector2::new(-pivot.x, -pivot.y));
        let moved = m.transform_point(pivot);
        assert!(
            close(moved.x, pivot.x) && close(moved.y, pivot.y),
            "{moved:?}"
        );

        // The box around it still contains the pivot, even though the axis-aligned
        // bounds cannot put a corner back on it.
        let r = Rectangle::new(10.0, 20.0, 30.0, 40.0);
        let b = m.transform_rect_bounds(r);
        assert!(
            pivot.x >= b.x - 1.0e-4
                && pivot.x <= b.x + b.width + 1.0e-4
                && pivot.y >= b.y - 1.0e-4
                && pivot.y <= b.y + b.height + 1.0e-4,
            "pivot {pivot:?} outside {b:?}"
        );
    }

    #[test]
    fn a_rigid_transform_reports_no_scale() {
        assert!(close(Matrix3::identity().scale_factor(), 1.0));
        assert!(close(Matrix3::rotation(1.234).scale_factor(), 1.0));
        assert!(close(
            Matrix3::translation(Vector2::new(9.0, -4.0)).scale_factor(),
            1.0
        ));
    }

    #[test]
    fn a_uniform_scale_reports_itself() {
        assert!(close(Matrix3::uniform_scale(4.0).scale_factor(), 4.0));
        assert!(close(Matrix3::uniform_scale(0.25).scale_factor(), 0.25));
    }

    #[test]
    fn scale_survives_a_rotation_composed_with_it() {
        let m = Matrix3::rotation(0.9) * Matrix3::uniform_scale(3.0);
        assert!(close(m.scale_factor(), 3.0), "was {}", m.scale_factor());
    }

    #[test]
    fn a_non_uniform_scale_reports_its_geometric_mean() {
        // 4x by 1x has an area 4x the original, so a linear factor of 2.
        assert!(close(
            Matrix3::scale(Vector2::new(4.0, 1.0)).scale_factor(),
            2.0
        ));
    }

    #[test]
    fn a_mirror_does_not_produce_a_negative_scale() {
        assert!(close(
            Matrix3::scale(Vector2::new(-3.0, 3.0)).scale_factor(),
            3.0
        ));
    }

    #[test]
    fn a_scaled_translation_is_scaled_too() {
        // This is the easy mistake: composing scale * translate scales the offset,
        // because the offset goes through the scale on its way out.
        let m = Matrix3::uniform_scale(2.0) * Matrix3::translation(Vector2::new(10.0, 0.0));
        assert_point(&m, Vector2::new(0.0, 0.0), Vector2::new(20.0, 0.0));
    }

    #[test]
    fn scale_translation_moves_by_more_without_resizing() {
        let m = Matrix3::translation(Vector2::new(10.0, -4.0)).scale_translation(2.0);
        assert_point(&m, Vector2::new(1.0, 1.0), Vector2::new(21.0, -7.0));
        assert!(
            close(m.scale_factor(), 1.0),
            "the linear part should be untouched, was {}",
            m.scale_factor()
        );
    }

    #[test]
    fn scale_translation_leaves_the_factors_alone() {
        // The mistake this guards against is scaling the whole matrix: a
        // `scale(2)` is a factor and not a length, so doubling it would change
        // what the element looks like rather than how many pixels it covers.
        let scaled = Matrix3::uniform_scale(3.0).scale_translation(2.0);
        assert!(
            close(scaled.scale_factor(), 3.0),
            "was {}",
            scaled.scale_factor()
        );
    }

    #[test]
    fn scale_translation_scales_a_composed_offset() {
        // `rotate * translate(10, 0)` puts the offset in the second column, and
        // ten logical pixels is twenty device pixels whatever direction the
        // element happens to face. Scaling the column is what expresses that.
        let rotated = Matrix3::rotation(0.5);
        let single = rotated * Matrix3::translation(Vector2::new(10.0, 0.0));
        let doubled = single.scale_translation(2.0);
        assert_point(
            &doubled,
            Vector2::new(0.0, 0.0),
            (rotated * Matrix3::translation(Vector2::new(20.0, 0.0)))
                .transform_point(Vector2::new(0.0, 0.0)),
        );
    }

    #[test]
    fn scale_translation_by_one_is_the_identity() {
        let m = Matrix3::rotation(0.9) * Matrix3::translation(Vector2::new(3.0, 4.0));
        assert_eq!(m.scale_translation(1.0), m);
    }
}
