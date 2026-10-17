#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::similar_names)]

pub use xs_math::{Rectangle, Vector2};

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

#[cfg(target_arch = "wasm32")]
pub mod layout_prof {
    pub use crate::flex::flex_layout::layout_prof::*;
}

mod image;
pub use image::*;

mod style;
pub use style::*;

mod text;
pub use text::*;

mod mesh_cache;
pub use mesh_cache::*;

mod vector;
pub use vector::*;

mod tree;
pub use tree::*;

mod widget;
pub use widget::*;
