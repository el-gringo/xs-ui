mod application;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();

pub use application::Application;
pub use xs_animation::{Animate, AnimationDirection, Easing, Spring};
pub use xs_core::{
    AnyElement, AppConfig, AppContext, Context, ElementObject, FocusHandle, IntoElement, KeyCode,
    NavigationNode, ParentElement, Render, State,
};
pub use xs_layout::*;
pub use xs_primitive::*;
// Named rather than globbed: `xs_vector` also has a `parse` - the one that reads
// a `d` attribute - and a document parser beside it is worth telling apart.
pub use xs_svg::{
    Svg, SvgError, SvgImage, SvgResult, parse as parse_svg, parse_bytes as parse_svg_bytes,
    parse_vector as parse_svg_vector,
};
pub use xs_vector::*;

pub mod prelude {
    pub use crate::{
        Animate, AnyElement, Color, Context, Easing, ElementObject, IntoElement, ParentElement,
        Render, Spring, WithAnimation, div, grid, parse_svg, parse_svg_vector, shape, text,
        vector,
    };
}
