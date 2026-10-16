//! SVG path data.
//!
//! The subset that draws: `M m L l H h V v C c S s Q q T t A a Z z`. That is all
//! of SVG 1.1's grammar, so any `d` attribute out there parses.
//!
//! Two places need care and both are grammar rather than interpretation:
//!
//! - **Flags are not numbers.** `A`'s `large-arc` and `sweep` are single
//!   characters, so `A 25 25 0 1050 0` means the flags are `1` and `0` followed by
//!   the point `50 0`. A parser that reads a flag as a number gets this right by
//!   accident; one that reads it as a number and a number gets it wrong.
//! - **A flag may be omitted entirely.** When the next character is a sign, that
//!   sign is the next number's, so the flag before it is implicitly `1`. That is
//!   why `read_flag` has an implicit case and why it is only the *second* flag
//!   that has one - the first always precedes a number, so it is always written.

use xs_math::Vector2;

use crate::path::{Path, PathError, Segment};

/// Reads SVG path data into a [`Path`].
///
/// Number parsing is forgiving in the two ways the grammar requires - a sign acts
/// as a separator, and exponent notation is accepted - and strict everywhere
/// else, so a typo is an error with an offset rather than a shape nobody meant.
pub fn parse(data: &str) -> Result<Path, PathError> {
    Parser::new(data).run()
}

/// Every command letter this parser knows.
const COMMANDS: [char; 10] = ['M', 'L', 'H', 'V', 'C', 'S', 'Q', 'T', 'A', 'Z'];

struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    /// The control point the next `S`/`T` reflects, and whether it came from a
    /// cubic - because `S` only reflects a cubic and `T` only a quadratic, and
    /// after the other kind each falls back to the current point.
    last_curve: Option<(Vector2, bool)>,
    current: Vector2,
    subpath_start: Vector2,
}

impl<'a> Parser<'a> {
    fn new(data: &'a str) -> Self {
        Self {
            bytes: data.as_bytes(),
            at: 0,
            last_curve: None,
            current: Vector2::zero(),
            subpath_start: Vector2::zero(),
        }
    }

    fn run(mut self) -> Result<Path, PathError> {
        let mut path = Path::new();
        self.skip_separators();
        // The raw letter of the command being read, case included. It has to be
        // the *raw* one because the `M` to `L` rewrite below would otherwise
        // lose whether the repeat was `M` or `m` - and with it whether the repeat
        // is relative.
        let mut letter: Option<char> = None;

        while self.at < self.bytes.len() {
            self.skip_separators();
            if self.at >= self.bytes.len() {
                break;
            }
            let command = if self.bytes[self.at].is_ascii_alphabetic() {
                let offset = self.at;
                let c = self.read_command()?;
                let upper = c.to_ascii_uppercase();
                if !COMMANDS.contains(&upper) {
                    return Err(PathError {
                        offset,
                        message: format!("unknown command '{c}'"),
                    });
                }
                letter = Some(c);
                upper
            } else {
                // Repeated arguments of the previous command. `M` is the one
                // exception SVG makes: a repeat is a line-to, not a move.
                match letter {
                    None => {
                        return Err(PathError {
                            offset: self.at,
                            message: String::from("expected a command"),
                        });
                    }
                    Some(previous) => match previous.to_ascii_uppercase() {
                        'M' => 'L',
                        upper => upper,
                    },
                }
            };
            let relative = letter.is_some_and(|c| c.is_ascii_lowercase());
            self.read_arguments(&mut path, command, relative)?;
        }
        Ok(path)
    }

    /// Whether the last command was lower case, which is what makes its
    /// arguments relative. Tracked from the raw letter because the `M` to `L`
    /// rewrite above turns `m` into `L` while the relativeness stays.
    fn read_command(&mut self) -> Result<char, PathError> {
        let c = self.bytes[self.at] as char;
        if !c.is_ascii_alphabetic() {
            return Err(PathError {
                offset: self.at,
                message: format!("expected a command, found '{c}'"),
            });
        }
        self.at += 1;
        Ok(c)
    }

    fn read_arguments(
        &mut self,
        path: &mut Path,
        command: char,
        relative: bool,
    ) -> Result<(), PathError> {
        match command {
            'M' => {
                let (x, y) = self.read_point()?;
                let target = self.resolve(x, y, relative);
                path.push(Segment::MoveTo(target));
                self.current = target;
                self.subpath_start = target;
                self.last_curve = None;
            }
            'L' => {
                let (x, y) = self.read_point()?;
                let target = self.resolve(x, y, relative);
                path.push(Segment::LineTo(target));
                self.current = target;
                self.last_curve = None;
            }
            'H' => {
                let x = self.read_number()?;
                let target = if relative { self.current.x + x } else { x };
                self.line_to(path, target, self.current.y);
            }
            'V' => {
                let y = self.read_number()?;
                let target = if relative { self.current.y + y } else { y };
                self.line_to(path, self.current.x, target);
            }
            'C' => {
                let c1 = self.read_point()?;
                let c2 = self.read_point()?;
                let to = self.read_point()?;
                let [c1, c2, to] = self.resolve_all([c1, c2, to], relative);
                path.push(Segment::CubicTo { c1, c2, to });
                self.last_curve = Some((c2, true));
                self.current = to;
            }
            'S' => {
                let c2 = self.read_point()?;
                let to = self.read_point()?;
                // Only a cubic reflects for an `S`, and only a quadratic for a
                // `T`. After the other kind - or after anything at all - SVG says
                // the control point is the current point, which makes the smooth
                // command an ordinary one.
                let c1 = match self.last_curve {
                    Some((control, was_cubic)) if was_cubic => Vector2::new(
                        2.0 * self.current.x - control.x,
                        2.0 * self.current.y - control.y,
                    ),
                    _ => self.current,
                };
                let [c2, to] = self.resolve_all([c2, to], relative);
                path.push(Segment::CubicTo { c1, c2, to });
                self.last_curve = Some((c2, true));
                self.current = to;
            }
            'Q' => {
                let control = self.read_point()?;
                let to = self.read_point()?;
                let [control, to] = self.resolve_all([control, to], relative);
                path.push(Segment::QuadraticTo { control, to });
                self.last_curve = Some((control, false));
                self.current = to;
            }
            'T' => {
                let to = self.read_point()?;
                let control = match self.last_curve {
                    Some((control, was_cubic)) if !was_cubic => Vector2::new(
                        2.0 * self.current.x - control.x,
                        2.0 * self.current.y - control.y,
                    ),
                    _ => self.current,
                };
                let [to] = self.resolve_all([to], relative);
                path.push(Segment::QuadraticTo { control, to });
                self.last_curve = Some((control, false));
                self.current = to;
            }
            'A' => {
                let rx = self.read_number()?;
                let ry = self.read_number()?;
                let rotation = self.read_number()?;
                let large_arc = self.read_flag()?;
                // The second flag may be left out entirely when a sign follows,
                // because then that sign belongs to the next number.
                let sweep = if self.peek_sign() {
                    true
                } else {
                    self.read_flag()?
                };
                let (x, y) = self.read_point()?;
                let to = self.resolve(x, y, relative);
                path.push(Segment::ArcTo {
                    radius: Vector2::new(rx, ry),
                    rotation: rotation.to_radians(),
                    large_arc,
                    sweep,
                    to,
                });
                self.current = to;
                // An arc has no control point for a following smooth command to
                // mirror.
                self.last_curve = None;
            }
            'Z' => {
                path.push(Segment::Close);
                self.current = self.subpath_start;
                // Nothing carries across the corner, so a following smooth command
                // starts from the current point rather than reflecting.
                self.last_curve = None;
            }
            // `run` checks the letter against `COMMANDS` before getting here.
            _ => unreachable!("command was validated before dispatch"),
        }
        Ok(())
    }

    fn line_to(&mut self, path: &mut Path, x: f32, y: f32) {
        let target = Vector2::new(x, y);
        path.push(Segment::LineTo(target));
        self.current = target;
        self.last_curve = None;
    }

    fn resolve(&self, x: f32, y: f32, relative: bool) -> Vector2 {
        if relative {
            Vector2::new(self.current.x + x, self.current.y + y)
        } else {
            Vector2::new(x, y)
        }
    }

    fn resolve_all<const N: usize>(&self, points: [(f32, f32); N], relative: bool) -> [Vector2; N] {
        let mut resolved = [Vector2::zero(); N];
        for (slot, (x, y)) in points.iter().enumerate() {
            resolved[slot] = self.resolve(*x, *y, relative);
        }
        resolved
    }

    /// One `x y` pair.
    fn read_point(&mut self) -> Result<(f32, f32), PathError> {
        let x = self.read_number()?;
        let y = self.read_number()?;
        Ok((x, y))
    }

    /// One number. A sign or an exponent is its own separator, which is why the
    /// separator skip happens *inside* the number and not only between arguments.
    fn read_number(&mut self) -> Result<f32, PathError> {
        self.skip_separators();
        let start = self.at;
        if self.peek_sign() {
            self.at += 1;
        }
        let mut saw_digit = false;
        while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
            self.at += 1;
            saw_digit = true;
        }
        if self.at < self.bytes.len() && self.bytes[self.at] == b'.' {
            self.at += 1;
            while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
                self.at += 1;
                saw_digit = true;
            }
        }
        if !saw_digit {
            self.at = start;
            return Err(PathError {
                offset: start,
                message: String::from("expected a number"),
            });
        }
        if self.at < self.bytes.len() && (self.bytes[self.at] | 0x20) == b'e' {
            let exponent_start = self.at;
            self.at += 1;
            if self.peek_sign() {
                self.at += 1;
            }
            let digits_start = self.at;
            while self.at < self.bytes.len() && self.bytes[self.at].is_ascii_digit() {
                self.at += 1;
            }
            if self.at == digits_start {
                // Not an exponent after all - `1e` followed by another command.
                // Rewind so the caller reports the `e` rather than swallowing it.
                self.at = exponent_start;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.at]).unwrap_or_default();
        text.parse::<f32>().map_err(|_| PathError {
            offset: start,
            message: format!("'{text}' is not a number"),
        })
    }

    fn peek_sign(&self) -> bool {
        matches!(self.bytes.get(self.at), Some(b'+') | Some(b'-'))
    }

    /// An arc flag: one character, and never more.
    fn read_flag(&mut self) -> Result<bool, PathError> {
        self.skip_separators();
        match self.bytes.get(self.at) {
            Some(b'0') => {
                self.at += 1;
                Ok(false)
            }
            Some(b'1') => {
                self.at += 1;
                Ok(true)
            }
            _ => Err(PathError {
                offset: self.at,
                message: String::from("expected an arc flag, 0 or 1"),
            }),
        }
    }

    fn skip_separators(&mut self) {
        while self.at < self.bytes.len()
            && matches!(self.bytes[self.at], b' ' | b'\t' | b'\n' | b'\r' | b',')
        {
            self.at += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_path_data_is_an_empty_path() {
        assert!(Path::parse("").unwrap().is_empty());
        assert!(Path::parse("   ").unwrap().is_empty());
        assert!(Path::parse("\n\t ").unwrap().is_empty());
    }

    #[test]
    fn moveto_and_lineto_absolutely() {
        let path = Path::parse("M10 20 L30 40").unwrap();
        assert_eq!(
            path.segments,
            vec![
                Segment::MoveTo(Vector2::new(10.0, 20.0)),
                Segment::LineTo(Vector2::new(30.0, 40.0)),
            ]
        );
    }

    #[test]
    fn relative_commands_are_relative_to_the_current_point() {
        let path = Path::parse("m10 10 l5 5 l5 -5").unwrap();
        assert_eq!(
            path.segments,
            vec![
                Segment::MoveTo(Vector2::new(10.0, 10.0)),
                Segment::LineTo(Vector2::new(15.0, 15.0)),
                Segment::LineTo(Vector2::new(20.0, 10.0)),
            ]
        );
    }

    /// The rule that catches everyone: a repeated argument set after `M` is a
    /// line-to, not another move.
    #[test]
    fn a_repeated_moveto_argument_is_a_lineto() {
        let path = Path::parse("M10 10 20 20 30 30").unwrap();
        assert_eq!(
            path.segments,
            vec![
                Segment::MoveTo(Vector2::new(10.0, 10.0)),
                Segment::LineTo(Vector2::new(20.0, 20.0)),
                Segment::LineTo(Vector2::new(30.0, 30.0)),
            ]
        );
    }

    /// ...and the relative form has to stay relative through that rewrite, which
    /// is the one place where remapping `m` to `L` can quietly go absolute.
    #[test]
    fn a_repeated_relative_moveto_argument_is_a_relative_lineto() {
        let path = Path::parse("m10 10 5 5").unwrap();
        assert_eq!(path.segments[1], Segment::LineTo(Vector2::new(15.0, 15.0)));
    }

    #[test]
    fn a_repeated_lineto_argument_stays_a_lineto() {
        let path = Path::parse("M0 0 L1 1 2 2 3 3").unwrap();
        assert_eq!(path.segments.len(), 4);
        assert_eq!(path.segments[3], Segment::LineTo(Vector2::new(3.0, 3.0)));
    }

    #[test]
    fn horizontal_and_vertical_keep_the_other_coordinate() {
        let path = Path::parse("M5 7 H30 V40 h-10 v-5").unwrap();
        assert_eq!(path.segments[1], Segment::LineTo(Vector2::new(30.0, 7.0)));
        assert_eq!(path.segments[2], Segment::LineTo(Vector2::new(30.0, 40.0)));
        assert_eq!(path.segments[3], Segment::LineTo(Vector2::new(20.0, 40.0)));
        assert_eq!(path.segments[4], Segment::LineTo(Vector2::new(20.0, 35.0)));
    }

    #[test]
    fn a_cubic_reads_all_three_points() {
        let path = Path::parse("M0 0 C1 2 3 4 5 6").unwrap();
        assert_eq!(
            path.segments[1],
            Segment::CubicTo {
                c1: Vector2::new(1.0, 2.0),
                c2: Vector2::new(3.0, 4.0),
                to: Vector2::new(5.0, 6.0),
            }
        );
    }

    #[test]
    fn a_smooth_cubic_mirrors_the_previous_second_control_point() {
        let path = Path::parse("M0 0 C1 2 3 4 5 6 S8 10 12 12").unwrap();
        let Segment::CubicTo { c1, c2, to } = path.segments[2] else {
            panic!("expected a cubic, got {:?}", path.segments[2]);
        };
        assert_eq!(c1, Vector2::new(7.0, 8.0));
        assert_eq!(c2, Vector2::new(8.0, 10.0));
        assert_eq!(to, Vector2::new(12.0, 12.0));
    }

    /// The reflection has to be of the *resolved* control point. Storing the raw
    /// one and mirroring it against an absolute endpoint is the bug this pins:
    /// the shape is right for absolute curves and drifts for relative ones.
    #[test]
    fn a_smooth_cubic_after_a_relative_cubic_reflects_the_right_point() {
        let path = Path::parse("m10 10 c1 2 3 4 5 6 s2 4 4 6").unwrap();
        let Segment::CubicTo { c1, c2, to } = path.segments[2] else {
            panic!("expected a cubic");
        };
        // The first cubic ran from (10,10) to (15,16) with c2 at (13,14), so the
        // mirror of (13,14) about (15,16) is (17,18).
        assert_eq!(c1, Vector2::new(17.0, 18.0));
        assert_eq!(c2, Vector2::new(17.0, 20.0));
        assert_eq!(to, Vector2::new(19.0, 22.0));
    }

    /// `S` and `T` are not interchangeable: each only reflects its own kind.
    #[test]
    fn a_smooth_cubic_after_a_quadratic_does_not_reflect() {
        let path = Path::parse("M0 0 Q1 2 3 4 S6 8 9 9").unwrap();
        let Segment::CubicTo { c1, .. } = path.segments[2] else {
            panic!("expected a cubic");
        };
        assert_eq!(c1, Vector2::new(3.0, 4.0), "must be the current point");
    }

    #[test]
    fn a_smooth_quadratic_after_a_cubic_does_not_reflect() {
        let path = Path::parse("M0 0 C1 2 3 4 5 6 T9 9").unwrap();
        let Segment::QuadraticTo { control, .. } = path.segments[2] else {
            panic!("expected a quadratic");
        };
        assert_eq!(control, Vector2::new(5.0, 6.0), "must be the current point");
    }

    #[test]
    fn a_smooth_quadratic_after_a_quadratic_reflects() {
        let path = Path::parse("M0 0 Q1 2 3 4 T6 8").unwrap();
        let Segment::QuadraticTo { control, to } = path.segments[2] else {
            panic!("expected a quadratic");
        };
        assert_eq!(control, Vector2::new(5.0, 6.0));
        assert_eq!(to, Vector2::new(6.0, 8.0));
    }

    #[test]
    fn a_smooth_command_after_a_moveto_starts_at_the_moveto() {
        let path = Path::parse("M7 8 S1 2 3 4").unwrap();
        let Segment::CubicTo { c1, .. } = path.segments[1] else {
            panic!("expected a cubic");
        };
        assert_eq!(c1, Vector2::new(7.0, 8.0));
    }

    /// Nothing carries across a close: there is no outgoing control point on the
    /// far side of a corner, so the next smooth command starts fresh.
    #[test]
    fn a_smooth_command_after_a_close_starts_at_the_corner() {
        let path = Path::parse("M0 0 Q1 2 3 4 Z T9 9").unwrap();
        let Segment::QuadraticTo { control, .. } = path.segments[3] else {
            panic!("expected a quadratic");
        };
        assert_eq!(control, Vector2::new(0.0, 0.0));
    }

    #[test]
    fn an_arc_reads_its_radius_rotation_and_flags() {
        let path = Path::parse("M0 0 A25 25 0 1 0 50 0").unwrap();
        let Segment::ArcTo {
            radius,
            rotation,
            large_arc,
            sweep,
            to,
        } = path.segments[1]
        else {
            panic!("expected an arc");
        };
        assert_eq!(radius, Vector2::new(25.0, 25.0));
        assert_eq!(rotation, 0.0);
        assert!(large_arc);
        assert!(!sweep);
        assert_eq!(to, Vector2::new(50.0, 0.0));
    }

    /// Flags are single characters, so `1050` is the flag `1`, the flag `0`, and
    /// then `50`. A parser that reads a flag as a number reads `1050` and then
    /// runs off the end.
    #[test]
    fn arc_flags_run_together_when_they_can() {
        let path = Path::parse("M0 0 A25 25 0 1050 0").unwrap();
        let Segment::ArcTo {
            large_arc,
            sweep,
            to,
            ..
        } = path.segments[1]
        else {
            panic!("expected an arc");
        };
        assert!(large_arc);
        assert!(!sweep);
        assert_eq!(to, Vector2::new(50.0, 0.0));
    }

    /// A flag followed immediately by a sign means the flag is implicitly `1` and
    /// the sign belongs to the next number. This is the `0-50` case and it is the
    /// reason `read_flag` is not enough on its own.
    #[test]
    fn an_arc_flag_may_be_implicit_before_a_signed_number() {
        let path = Path::parse("M0 0 A25 25 10 0-50 0").unwrap();
        let Segment::ArcTo {
            rotation,
            large_arc,
            sweep,
            to,
            ..
        } = path.segments[1]
        else {
            panic!("expected an arc");
        };
        assert!((rotation.to_degrees() - 10.0).abs() < 1.0e-4);
        assert!(!large_arc);
        assert!(sweep, "the omitted flag before a sign is implicitly 1");
        assert_eq!(to, Vector2::new(-50.0, 0.0));
    }

    #[test]
    fn an_arc_rotation_is_in_degrees_and_stored_in_radians() {
        let path = Path::parse("M0 0 A5 5 90 0 0 10 10").unwrap();
        let Segment::ArcTo { rotation, .. } = path.segments[1] else {
            panic!("expected an arc");
        };
        assert!((rotation.to_degrees() - 90.0).abs() < 1.0e-4);
    }

    #[test]
    fn close_returns_to_the_start_of_its_own_contour() {
        let path = Path::parse("M0 0 L10 0 M20 20 L30 20 Z L5 5").unwrap();
        assert_eq!(path.segments[4], Segment::Close);
        // A line-to after a close starts from the *closed contour's* start, not
        // from the point before the close - which is what makes
        // `M0 0 h10 v10 z l-10 -10` a rectangle.
        assert_eq!(path.segments[5], Segment::LineTo(Vector2::new(5.0, 5.0)));
    }

    #[test]
    fn numbers_may_use_exponents_and_a_leading_dot() {
        let path = Path::parse("M.5 1e2 L-1.5E-1 2").unwrap();
        assert_eq!(path.segments[0], Segment::MoveTo(Vector2::new(0.5, 100.0)));
        assert_eq!(path.segments[1], Segment::LineTo(Vector2::new(-0.15, 2.0)));
    }

    #[test]
    fn a_sign_is_its_own_separator() {
        let path = Path::parse("M0 0 1-2 3-4").unwrap();
        assert_eq!(path.segments[1], Segment::LineTo(Vector2::new(1.0, -2.0)));
        assert_eq!(path.segments[2], Segment::LineTo(Vector2::new(3.0, -4.0)));
    }

    #[test]
    fn newlines_and_commas_are_separators() {
        let path = Path::parse("M 0,0\n L 1,1\n\tL2,2").unwrap();
        assert_eq!(path.segments.len(), 3);
        assert_eq!(path.segments[2], Segment::LineTo(Vector2::new(2.0, 2.0)));
    }

    /// Each `l` moves from the last point, not from the subpath start.
    #[test]
    fn a_relative_command_is_relative_to_the_point_before_it() {
        let path = Path::parse("M10 10 l5 0 l0 5 l-5 0 z").unwrap();
        assert_eq!(path.segments[1], Segment::LineTo(Vector2::new(15.0, 10.0)));
        assert_eq!(path.segments[2], Segment::LineTo(Vector2::new(15.0, 15.0)));
        assert_eq!(path.segments[3], Segment::LineTo(Vector2::new(10.0, 15.0)));
    }

    #[test]
    fn an_unknown_command_reports_the_offset_of_the_letter() {
        let error = Path::parse("M0 0 X5 5").unwrap_err();
        assert_eq!(error.offset, 5);
        assert!(error.message.contains('X'), "{}", error.message);
    }

    #[test]
    fn a_path_that_does_not_start_with_a_command_is_an_error() {
        // Note that `M0 0 5 5` is *not* this case: it is a move and a line-to,
        // which is the rule `a_repeated_moveto_argument_is_a_lineto` pins. A path
        // has to open with a command.
        let error = Path::parse("5 5 L10 10").unwrap_err();
        assert!(error.message.contains("command"), "{}", error.message);
    }

    /// ...but a repeat after a command is not an error, so this one has to parse.
    #[test]
    fn a_repeat_starts_where_the_last_command_left_off() {
        let path = Path::parse("M0 0 5 5").unwrap();
        assert_eq!(path.segments[1], Segment::LineTo(Vector2::new(5.0, 5.0)));
    }

    #[test]
    fn missing_arguments_are_an_error_rather_than_a_guess() {
        let error = Path::parse("M0 0 L10").unwrap_err();
        assert!(error.message.contains("number"), "{}", error.message);
    }

    #[test]
    fn a_non_flag_arc_argument_is_an_error() {
        let error = Path::parse("M0 0 A5 5 0 2 0 10 10").unwrap_err();
        assert!(error.message.contains("flag"), "{}", error.message);
    }

    #[test]
    fn a_command_with_no_arguments_at_all_is_an_error() {
        assert!(Path::parse("M0 0 Z Z q").is_err());
    }

    #[test]
    fn a_lone_e_is_not_swallowed_as_an_exponent() {
        // `1e` must not parse as `1` with a stray `e`; the error has to name it.
        let error = Path::parse("M1e 2").unwrap_err();
        assert!(error.message.contains('e'), "{}", error.message);
    }

    /// Everything a real icon font or a real drawing program emits, in one go. A
    /// parser that handles the grammar but not real shapes is still useless.
    #[test]
    fn a_realistic_icon_path_parses() {
        let data = "M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 \
                    2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8z";
        let path = Path::parse(data).unwrap();
        assert!(path.point_count() > 8, "only {} points", path.point_count());
        assert!(!path.is_empty());
    }
}
