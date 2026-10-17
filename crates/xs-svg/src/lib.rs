//! SVG in, [`xs_vector::Shape`]s out.
//!
//! An SVG document is read with [`xs_xml`] into a list of shapes that
//! [`xs_vector::Vector`] draws, which is what an icon is: a few paths, in the
//! document's own units, scaled into whatever box layout gives the node.
//!
//! ```
//! use xs_svg::parse_vector;
//!
//! let icon = parse_vector(
//!     r#"<svg viewBox="0 0 24 24"><path d="M4 4 L20 20" stroke="red" stroke-width="2"/></svg>"#,
//! )
//! .unwrap();
//!
//! assert_eq!(icon.shapes.len(), 1);
//! let stroke = icon.shapes[0].stroke.as_ref().unwrap();
//! assert_eq!(stroke.width, 2.0);
//! ```
//!
//! # What it reads
//!
//! * **Shapes**: `path`, `rect` with `rx`/`ry`, `circle`, `ellipse`, `line`,
//!   `polyline`, `polygon`.
//! * **Structure**: `svg`, `g`, `a`, `switch`, nested `svg`, and `defs` with
//!   `use` - including a `use` that points forwards at an `id` defined later.
//! * **Transforms**: `transform` on any element, as `translate`, `scale`,
//!   `rotate` (with or without a centre), `skewX`, `skewY` and `matrix`,
//!   composed the way SVG composes a list.
//! * **Paint**: `fill`, `stroke`, `stroke-width`, `stroke-linecap`,
//!   `stroke-dasharray`, `fill-opacity`, `stroke-opacity`, `opacity` and
//!   `color`/`currentColor`, each from a presentation attribute or from a
//!   `style` declaration, inherited down the tree.
//! * **Gradients**: `linearGradient` with its `x1`/`y1`/`x2`/`y2`, resolved
//!   through a `fill="url(#id)"` from anywhere in the document. The two end
//!   stops are the ramp; `stop-opacity` multiplies into its own stop.
//! * **Colours**: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`, `rgba()`,
//!   `hsl()`, `hsla()`, the 147 SVG keywords, `transparent`, `currentColor`
//!   and `none`.
//! * **Sizes**: `viewBox`, `width` and `height`, with the CSS absolute units
//!   and percentages resolved against the view box.
//!
//! # What it does not
//!
//! Each of these is **skipped**, not an error, because a real file is full of
//! features that have nothing to do with what ends up on screen:
//!
//! * `gradientUnits`, `spreadMethod`, a colour stop in the middle of a ramp
//!   (the first and last are the ramp), radial gradients, and `href` from one
//!   gradient to another.
//! * `clipPath`, `mask`, `pattern`, `filter`, `marker`. A
//!   `fill="url(#not-a-gradient)"` is not an error either: the shape falls
//!   back to the paint it inherits.
//! * CSS. A `<style>` block and `class` selectors are not read; presentation
//!   attributes and inline `style` are.
//! * Text, `image`, `foreignObject`, and every animation element.
//! * `switch`'s conditionals: every child is drawn, where a browser would draw
//!   the first whose conditions hold.
//! * `preserveAspectRatio`: the view box is applied by the node, at its default
//!   `xMidYMid meet`.
//! * Namespace resolution: `svg:rect` is read as `rect`, and a prefixed name is
//!   left prefixed.
//!
//! An element that is drawn but has no geometry - `<path/>` with no `d` - is
//! skipped too. What *is* an error is a value that says one thing and means
//! another: a `fill` that is not a colour, a `viewBox` that is not four numbers,
//! a `d` that is not path data, or a `<use>` whose reference resolves to nothing.
//!
//! # One approximation
//!
//! `opacity` on a group composites it offscreen, so two translucent children
//! overlap and the overlap is darker. Nothing here has an offscreen target, so
//! each shape carries the product of the opacities above it instead. For icons -
//! what this crate is for - the two agree, because icons rarely overlap. See
//! [`SvgImage`].
//!
//! # Errors
//!
//! The failures worth naming, all of which carry the attribute they are about:
//! [`SvgError::NotSvg`], [`SvgError::BadValue`], [`SvgError::Path`],
//! [`SvgError::UnknownReference`] and [`SvgError::RecursiveUse`].

#![warn(clippy::all, clippy::pedantic)]

mod cached;
mod color;
mod error;
mod geometry;
mod gradient;
mod image;
mod paint;
mod transform;
mod values;

pub use cached::Svg;
pub use error::{SvgError, SvgResult};
pub use gradient::Gradients;
pub use image::{SvgImage, parse, parse_bytes, parse_vector};
pub use paint::Paint;

/// The paint value reader, for an app that resolves `currentColor` itself.
pub mod colors {
    pub use crate::color::parse;
}

/// The `transform` attribute reader, for an app that composes its own.
pub mod transforms {
    pub use crate::transform::parse;
}
