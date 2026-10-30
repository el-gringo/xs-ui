//! A small, `react-router`-shaped router for `xs-ui`.
//!
//! The route tree is declared as data, matched in priority order, and rendered through
//! the normal [`xs_core::Render`] trait. On wasm the history is the browser's
//! own: navigation calls `history.pushState`/`replaceState`, and the back and
//! forward buttons arrive as `popstate` events. On native - and in tests - the
//! same API runs on an in-memory history.
//!
//! ```ignore
//! use xs::prelude::*;
//!
//! fn main() {
//!     Application::new().run(|_ctx| {
//!         router(
//!             route("/", layout)
//!                 .child(index(home))
//!                 .child(route("users", users))
//!                 .child(route("*", not_found)),
//!         )
//!     });
//! }
//! ```
//!
//! [`route`], [`index`], [`layout`] and [`router`] are the constructors,
//! equivalent to [`Route::new`], [`Route::index`], [`Route::layout`] and
//! [`Router::new`]. A route component is any zero-argument function whose
//! result turns into an [`xs_core::AnyElement`], which includes a [`Render`]
//! type and an already-built element.
//!
//! A route that has children renders them through [`outlet`]. The component
//! needs the render context for that, so it is a [`Render`] type rather than a
//! closure:
//!
//! ```ignore
//! struct Users;
//! impl Render for Users {
//!     fn render(&self, ctx: &mut Context) -> impl IntoElement {
//!         div().child(outlet(ctx))
//!     }
//! }
//!
//! route("users", users)
//!     .child(index(all_users))
//!     .child(route(":id", user))
//! ```
//!
//! [`Route::layout`] is only a shared path prefix: it has no component, so it
//! draws nothing and needs no outlet.
//!
//! The router provides each [`RouteContext`] into the render context, so a
//! component deep inside a subtree can read it with [`use_route`] instead of
//! threading it through every argument:
//!
//! ```ignore
//! struct UserRow { id: usize }
//!
//! impl Render for UserRow {
//!     fn render(&self, ctx: &mut Context) -> impl IntoElement {
//!         let router = use_router(ctx);
//!         div().child(text(self.id.to_string()))
//!     }
//! }
//! ```
//!
//! The browser has no pointer events wired up yet, so a [`Link`] is activated
//! programmatically (`link(ctx, "/users").activate()`) or from a focus handler.

mod history;
mod link;
mod location;
mod params;
mod pattern;
mod route;
mod router;
mod routing;

pub use history::{History, MemoryHistory};
pub use link::{Link, link};
pub use location::Location;
pub use params::Params;
pub use route::{
    Outlet, Route, RouteComponent, RouteContext, index, layout, outlet, route, use_location,
    use_params, use_route, use_router,
};
pub use router::{Router, RouterHandle, router};

#[cfg(target_arch = "wasm32")]
pub use history::BrowserHistory;
