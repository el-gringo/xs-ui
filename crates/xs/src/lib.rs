mod application;

#[cfg(target_arch = "wasm32")]
#[global_allocator]
static ALLOC: talc::wasm::WasmDynamicTalc = talc::wasm::new_wasm_dynamic_allocator();

pub use application::Application;
pub use xs_animation::{Animate, AnimationDirection, Easing, Spring};
pub use xs_core::{
    AnyElement, AppConfig, AppContext, Context, ElementObject, FocusHandle, IntoElement, KeyCode,
    NavigationNode, ParentElement, Render, Setter, State,
};
pub use xs_layout::*;
pub use xs_primitive::*;

pub mod prelude {
    pub use crate::{
        Animate, AnyElement, Color, Context, Easing, ElementObject, IntoElement, ParentElement,
        Render, Spring, WithAnimation, div, grid, text,
    };
}
