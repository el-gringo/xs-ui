//! `<linearGradient>`, and the `fill="url(#id)"` that reaches for it.
//!
//! A linear gradient is two colours and a direction, which is what
//! [`LinearGradient`] carries and what the quad shader already does for a box -
//! so a gradient in a shape and a gradient on a `Div` are the same feature, not
//! two that look alike.
//!
//! What is **not** read, and why it is safe to say so: `gradientUnits`,
//! `spreadMethod`, a radial gradient, a stop in the middle of the ramp, and an
//! `href` from one gradient to another. Every one of those changes where the
//! colours land rather than whether there are two of them, so a file using them
//! still draws - with the first and last stops along the direction its own `x1`
//! `y1` `x2` `y2` give.

use std::collections::HashMap;

use xs_layout::Color;
use xs_vector::LinearGradient;
use xs_xml::Element;

use crate::geometry::local_name;
use crate::values::number;

/// Every gradient in a document, by id.
#[derive(Clone, Debug, Default)]
pub struct Gradients {
    by_id: HashMap<String, LinearGradient>,
}

impl Gradients {
    /// Reads the gradients out of a tree.
    ///
    /// A whole tree rather than one element, because a gradient is usually in
    /// `<defs>` and the path that uses it is nowhere near it.
    #[must_use]
    pub fn collect(root: &Element) -> Self {
        let mut by_id = HashMap::new();
        Self::walk(root, &mut by_id);
        Self { by_id }
    }

    fn walk(element: &Element, by_id: &mut HashMap<String, LinearGradient>) {
        if local_name(element.name()) == "linearGradient"
            && let Some(id) = element.attribute("id")
        {
            by_id.insert(id.to_owned(), read(element));
        }
        for child in element.children() {
            if let Some(child) = child.as_element() {
                Self::walk(child, by_id);
            }
        }
    }

    /// The gradient a `url(#id)` refers to.
    ///
    /// `None` for a reference to something that is not here - a gradient in
    /// another file, or one this parser skipped - which is the same answer as a
    /// paint it could not read.
    #[must_use]
    pub fn get(&self, reference: &str) -> Option<LinearGradient> {
        let id = reference.trim().strip_prefix("url(#")?.strip_suffix(')')?;
        self.by_id.get(id).copied()
    }

    /// Every id read, for the tests and for a caller that wants to know.
    #[must_use]
    pub fn len(&self) -> usize {
        self.by_id.len()
    }

    /// Whether there are none.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.by_id.is_empty()
    }
}

/// One `<linearGradient>`: its direction, and its two end stops.
fn read(element: &Element) -> LinearGradient {
    // `x1 y1 x2 y2` are fractions of the box in the default units, so the
    // direction is the difference and the angle is that difference's, in the same
    // y-down space as everything else here.
    let x1 = coordinate(element, "x1", 0.0);
    let y1 = coordinate(element, "y1", 0.0);
    let x2 = coordinate(element, "x2", 1.0);
    let y2 = coordinate(element, "y2", 0.0);
    let angle = (y2 - y1).atan2(x2 - x1);

    // The first and last stop that carries a colour. A stop with an offset but no
    // colour is legal and is simply not a colour.
    let mut stops: Vec<Color> = Vec::new();
    for child in element.children() {
        let Some(stop) = child.as_element() else {
            continue;
        };
        if local_name(stop.name()) != "stop" {
            continue;
        }
        let Some(color) = stop.attribute("stop-color") else {
            continue;
        };
        let alpha = match stop.attribute("stop-opacity") {
            Some(value) => number(value, "stop-opacity").unwrap_or(1.0).clamp(0.0, 1.0),
            None => 1.0,
        };
        if let Ok(Some(mut color)) = crate::color::parse(color, None) {
            color.a *= alpha;
            stops.push(color);
        }
    }

    let start = stops.first().copied().unwrap_or(Color::BLACK);
    let end = stops.last().copied().unwrap_or(start);
    LinearGradient::new(start, end, angle)
}

fn coordinate(element: &Element, name: &'static str, fallback: f32) -> f32 {
    element
        .attribute(name)
        .and_then(|value| number(value, name).ok())
        .unwrap_or(fallback)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradients_of(document: &str) -> Gradients {
        let document = xs_xml::parse(document).expect("parses");
        Gradients::collect(document.root().expect("has a root"))
    }

    #[test]
    fn reads_a_vertical_gradient() {
        let gradients = gradients_of(
            r##"<svg><defs><linearGradient id="g" x1="0" x2="0" y1="0" y2="1">
                 <stop offset="0%" stop-color="#ff0000"/>
                 <stop offset="100%" stop-color="#0000ff"/>
               </linearGradient></defs></svg>"##,
        );
        let gradient = gradients.get("url(#g)").expect("it is there");
        assert_eq!(gradient.start, Color::RED);
        assert_eq!(gradient.end, Color::BLUE);
        let expected = std::f32::consts::FRAC_PI_2;
        assert!(
            (gradient.angle - expected).abs() < 1e-6,
            "down, in y-down space"
        );
    }

    #[test]
    fn reads_a_horizontal_gradient_from_its_own_direction() {
        let gradients = gradients_of(
            r##"<svg><linearGradient id="g" x1="0" y1="0" x2="1" y2="0">
                 <stop offset="0" stop-color="#000000"/>
                 <stop offset="1" stop-color="#ffffff"/>
               </linearGradient></svg>"##,
        );
        let gradient = gradients.get("url(#g)").expect("it is there");
        assert_eq!(gradient.angle, 0.0);
    }

    #[test]
    fn a_gradient_with_one_stop_is_that_colour_throughout() {
        let gradients = gradients_of(
            r#"<svg><linearGradient id="g"><stop offset="0" stop-color="red"/></linearGradient></svg>"#,
        );
        let gradient = gradients.get("url(#g)").expect("it is there");
        assert_eq!(gradient.start, Color::RED);
        assert_eq!(gradient.end, Color::RED);
    }

    #[test]
    fn a_stop_opacity_multiplies_into_the_colour() {
        let gradients = gradients_of(
            r##"<svg><linearGradient id="g">
                 <stop offset="0" stop-color="#ff0000" stop-opacity="0.5"/>
                 <stop offset="1" stop-color="#ff0000"/>
               </linearGradient></svg>"##,
        );
        let gradient = gradients.get("url(#g)").expect("it is there");
        assert!((gradient.start.a - 0.5).abs() < 1e-6);
        assert_eq!(gradient.end.a, 1.0);
    }

    #[test]
    fn a_reference_to_nothing_is_none_rather_than_an_error() {
        let gradients = gradients_of("<svg><rect/></svg>");
        assert_eq!(gradients.get("url(#missing)"), None);
        assert_eq!(gradients.get("not a reference"), None);
        assert!(gradients.is_empty());
        assert_eq!(gradients.len(), 0);
    }

    #[test]
    fn a_gradient_is_found_inside_defs_and_under_a_group() {
        let gradients = gradients_of(
            r##"<svg><defs><g><linearGradient id="deep" y2="1">
                 <stop offset="0" stop-color="#000000"/><stop offset="1" stop-color="#ffffff"/>
               </linearGradient></g></defs></svg>"##,
        );
        assert_eq!(gradients.len(), 1);
        assert!(gradients.get("url(#deep)").is_some());
    }
}
