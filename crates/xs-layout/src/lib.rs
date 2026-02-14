#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::similar_names)]

mod base;
pub use base::*;

mod color;
pub use color::*;

mod container;
pub use container::*;

mod decoration;
pub use decoration::*;

mod image;
pub use image::*;

mod tree;
pub use tree::*;

mod widget;
pub use widget::*;

mod flex;
pub use flex::*;
