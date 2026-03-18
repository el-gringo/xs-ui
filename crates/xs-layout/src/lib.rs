#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::similar_names)]

pub use xs_math::Vector2;

mod base;
pub use base::*;

mod color;
pub use color::*;

mod container;
pub use container::*;

mod decoration;
pub use decoration::*;

mod flex;
pub use flex::*;

mod image;
pub use image::*;

mod style;
pub use style::*;

mod text;
pub use text::*;

mod tree;
pub use tree::*;

mod widget;
pub use widget::*;
