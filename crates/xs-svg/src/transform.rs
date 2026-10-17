//! The `transform` attribute: a list of affine transforms.
//!
//! SVG composes a list left to right into a single matrix, which is what
//! [`Matrix3::then`] already does, so the parsing here is only about reading
//! `name(args)` after `name(args)` out of a string.

use xs_math::{Matrix3, Vector2};

use crate::error::{SvgError, SvgResult};
use crate::values::number;

/// Reads a whole transform list.
///
/// # Errors
///
/// [`SvgError::BadValue`] for an unknown function, an unclosed one, or an
/// argument that is not a number - each reported as the `name(args)` it was in.
///
/// `""` and a missing attribute are the identity, which is what
/// [`Matrix3::identity`] is: an element with no `transform` is not transformed.
pub fn parse(value: &str) -> SvgResult<Matrix3> {
    let chars: Vec<char> = value.chars().collect();
    let mut at = 0usize;
    let mut matrix = Matrix3::identity();

    while at < chars.len() {
        skip_separators(&chars, &mut at);
        if at >= chars.len() {
            break;
        }

        let start = at;
        while at < chars.len() && chars[at].is_ascii_alphabetic() {
            at += 1;
        }
        let name: String = chars[start..at].iter().collect();
        skip_separators(&chars, &mut at);
        if at >= chars.len() || chars[at] != '(' {
            return Err(SvgError::BadValue {
                attribute: "transform",
                value: value.to_owned(),
            });
        }
        at += 1;

        let start = at;
        while at < chars.len() && chars[at] != ')' {
            at += 1;
        }
        if at >= chars.len() {
            return Err(SvgError::BadValue {
                attribute: "transform",
                value: value.to_owned(),
            });
        }
        let arguments: String = chars[start..at].iter().collect();
        at += 1;

        matrix = matrix.then(&one(&name, &arguments)?);
    }

    Ok(matrix)
}

/// One `name(args)` of a transform list.
fn one(name: &str, arguments: &str) -> SvgResult<Matrix3> {
    let bad = || SvgError::BadValue {
        attribute: "transform",
        value: format!("{name}({arguments})"),
    };
    let values: Vec<f32> = arguments
        .split([' ', ',', '\t', '\n', '\r'])
        .filter(|token| !token.is_empty())
        .map(|token| number(token, "transform"))
        .collect::<SvgResult<_>>()?;

    // The two-argument forms and the one-argument forms differ only in that the
    // second argument repeats the first or defaults, so each arm reads the two
    // it needs rather than a fixed count.
    let get = |index: usize| values.get(index).copied().ok_or_else(bad);
    let get_or = |index: usize, fallback: f32| match values.get(index) {
        Some(value) => *value,
        None => fallback,
    };

    Ok(match name {
        "translate" => Matrix3::translation(Vector2::new(get(0)?, get_or(1, 0.0))),
        "scale" => {
            let x = get(0)?;
            Matrix3::scale(Vector2::new(x, get_or(1, x)))
        }
        "rotate" => {
            let degrees = get(0)?;
            // `rotate(a cx cy)` turns about `(cx, cy)` rather than the origin,
            // which is the translation either side of the rotation.
            let centre = Vector2::new(get_or(1, 0.0), get_or(2, 0.0));
            Matrix3::translation(centre)
                .then(&Matrix3::rotation(degrees.to_radians()))
                .then(&Matrix3::translation(-centre))
        }
        // `skewX` slides x by the tangent of y's angle, so the tangent is the
        // coefficient of y in the x column.
        "skewX" => Matrix3 {
            columns: [
                [1.0, 0.0, 0.0],
                [get(0)?.to_radians().tan(), 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        },
        "skewY" => Matrix3 {
            columns: [
                [1.0, get(0)?.to_radians().tan(), 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
        },
        // The arguments are the specification's own `a b c d e f`.
        "matrix" => {
            let values = six(&values, &bad)?;
            // `columns[column][0]` is that variable's coefficient in the x
            // output, so the four linear arguments are one per column and the
            // translation is the third.
            Matrix3 {
                columns: [
                    [values[0], values[1], 0.0],
                    [values[2], values[3], 0.0],
                    [values[4], values[5], 1.0],
                ],
            }
        }
        _ => return Err(bad()),
    })
}

/// The six arguments of `matrix()`, and exactly six: every other transform takes
/// one or two.
fn six(values: &[f32], bad: &impl Fn() -> SvgError) -> SvgResult<[f32; 6]> {
    match values.try_into() {
        Ok(values) => Ok(values),
        Err(_) => Err(bad()),
    }
}

fn skip_separators(chars: &[char], at: &mut usize) {
    while matches!(chars.get(*at), Some(c) if c.is_whitespace() || *c == ',') {
        *at += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point_of(value: &str) -> (f32, f32) {
        let point = parse(value)
            .expect("parses")
            .transform_point(Vector2::new(1.0, 0.0));
        (point.x, point.y)
    }

    #[test]
    fn an_empty_list_is_the_identity() {
        let expected = Matrix3::identity();
        assert_eq!(parse("").unwrap(), expected);
        assert_eq!(parse("  ").unwrap(), expected);
    }

    #[test]
    fn translates() {
        let expected = (11.0, 20.0);
        assert_eq!(point_of("translate(10,20)"), expected);
        let expected = (11.0, 0.0);
        assert_eq!(point_of("translate(10)"), expected);
    }

    #[test]
    fn scales_uniformly_unless_told_otherwise() {
        let expected = (2.0, 0.0);
        assert_eq!(point_of("scale(2)"), expected);
        let expected = (2.0, 0.0);
        assert_eq!(point_of("scale(2,2)"), expected);
    }

    #[test]
    fn scales_by_axis_when_given_two() {
        let matrix = parse("scale(2,3)").expect("parses");
        let point = matrix.transform_point(Vector2::new(1.0, 1.0));
        let expected = (2.0, 3.0);
        assert_eq!((point.x, point.y), expected);
    }

    #[test]
    fn rotates_about_the_origin_by_default() {
        let matrix = parse("rotate(90)").expect("parses");
        let point = matrix.transform_point(Vector2::new(1.0, 0.0));
        let expected = (0.0, 1.0);
        assert!((point.x - expected.0).abs() < 1e-5);
        assert!((point.y - expected.1).abs() < 1e-5);
    }

    #[test]
    fn rotates_about_a_point_when_given_one() {
        let matrix = parse("rotate(90 10 10)").expect("parses");
        let point = matrix.transform_point(Vector2::new(10.0, 10.0));
        // The centre does not move, which is the whole point of naming it.
        assert!((point.x - 10.0).abs() < 1e-5);
        assert!((point.y - 10.0).abs() < 1e-5);

        let matrix = parse("rotate(90, 10, 10)").expect("parses");
        let point = matrix.transform_point(Vector2::new(10.0, 10.0));
        assert!((point.x - 10.0).abs() < 1e-5);
    }

    #[test]
    fn skews() {
        let matrix = parse("skewX(45)").expect("parses");
        let point = matrix.transform_point(Vector2::new(0.0, 1.0));
        assert!((point.x - 1.0).abs() < 1e-5);
        assert!((point.y - 1.0).abs() < 1e-5);

        let matrix = parse("skewY(45)").expect("parses");
        let point = matrix.transform_point(Vector2::new(1.0, 0.0));
        assert!((point.y - 1.0).abs() < 1e-5);
    }

    #[test]
    fn reads_a_matrix() {
        let matrix = parse("matrix(1 0 0 1 10 20)").expect("parses");
        assert_eq!([matrix.columns[2][0], matrix.columns[2][1]], [10.0, 20.0]);

        // A scale and a skew, which is what the four leading arguments are for.
        let matrix = parse("matrix(2 0 0 3 0 0)").expect("parses");
        let point = matrix.transform_point(Vector2::new(1.0, 1.0));
        let expected = (2.0, 3.0);
        assert_eq!((point.x, point.y), expected);
    }

    #[test]
    fn a_list_composes_left_to_right() {
        // `translate(10,0) scale(2)` moves the point by 10 and then doubles it,
        // which is the order the list is written in.
        let matrix = parse("translate(10,0) scale(2)").expect("parses");
        let point = matrix.transform_point(Vector2::new(1.0, 0.0));
        let expected = (12.0, 0.0);
        assert_eq!((point.x, point.y), expected);
    }

    #[test]
    fn a_list_needs_no_separators() {
        let spaced = parse("translate(10,20) scale(2)").expect("parses");
        let tight = parse("translate(10 20)scale(2)").expect("parses");
        assert_eq!(spaced, tight);
    }

    #[test]
    fn an_unknown_transform_is_an_error() {
        let error = parse("wobble(3)").unwrap_err();
        let expected = "'wobble(3)' is not a valid transform";
        assert_eq!(error.to_string(), expected);
    }

    #[test]
    fn a_transform_with_the_wrong_arguments_is_an_error() {
        let error = parse("matrix(1 0 0)").unwrap_err();
        assert!(error.to_string().contains("is not a valid transform"));

        let error = parse("translate()").unwrap_err();
        assert!(error.to_string().contains("is not a valid transform"));
    }

    #[test]
    fn a_transform_with_a_non_number_is_an_error() {
        let error = parse("translate(10, far)").unwrap_err();
        assert!(error.to_string().contains("is not a valid transform"));
    }

    #[test]
    fn an_unclosed_transform_is_an_error() {
        let error = parse("translate(10,20").unwrap_err();
        assert!(error.to_string().contains("is not a valid transform"));
    }

    #[test]
    fn a_transform_with_no_parentheses_is_an_error() {
        let error = parse("translate").unwrap_err();
        assert!(error.to_string().contains("is not a valid transform"));
    }
}
