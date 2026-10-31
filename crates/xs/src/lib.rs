mod application;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();

pub use application::Application;
pub use xs_animation::{Animate, AnimationDirection, Easing, Spring};
pub use xs_core::{
    AnyElement, AppConfig, AppContext, Context, ContextValues, ElementObject, FocusHandle,
    IntoElement, KeyCode, NavigationLayout, NavigationNode, ParentElement, Render, State,
};
pub use xs_layout::*;
pub use xs_primitive::*;
pub use xs_router::*;
// Named rather than globbed: `xs_vector` also has a `parse` - the one that reads
// a `d` attribute - and a document parser beside it is worth telling apart.
pub use xs_svg::{
    Svg, SvgError, SvgImage, SvgResult, parse as parse_svg, parse_bytes as parse_svg_bytes,
    parse_vector as parse_svg_vector,
};
pub use xs_vector::*;

pub mod prelude {
    pub use crate::{
        Animate, AnyElement, Color, Context, ContextValues, Easing, ElementObject, IntoElement,
        Link, ParentElement, Render, Route, RouteContext, Router, Spring, WithAnimation, div, grid,
        index, layout, link, outlet, parse_svg, parse_svg_vector, route, router, shape, text,
        use_location, use_params, use_route, use_router, vector,
    };
}
