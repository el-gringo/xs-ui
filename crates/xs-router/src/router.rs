use std::{cell::RefCell, rc::Rc};

use xs_core::{AnyElement, Context, Render, State};

use crate::{
    Location, Route,
    history::{History, platform_history},
    route::{Rendered, RouteContext, empty},
    routing::{MatchedRoute, match_routes},
};

/// The router's shared state. One per mounted [`Router`].
pub(crate) struct RouterInner {
    pub(crate) history: Box<dyn History>,
    pub(crate) location: State<Location>,
    pub(crate) root: Route,
    /// The match chain of the last render, which is what [`crate::Outlet`]
    /// walks. Written before the root route renders and read while the tree
    /// resolves, both on the same thread and within one frame.
    pub(crate) matched: RefCell<Rc<Vec<MatchedRoute>>>,
}

impl RouterInner {
    pub(crate) fn render_match(
        &self,
        router: &RouterHandle,
        ctx: &mut Context,
        matched: &MatchedRoute,
        depth: usize,
        location: &Location,
    ) -> AnyElement {
        let route = RouteContext {
            router: router.clone(),
            params: matched.params.clone(),
            location: location.clone(),
            depth,
            path: matched.path.clone(),
        };
        // Provided rather than stored globally: the value lives in this
        // element's render context, so it reaches the component's whole subtree
        // and a nested route shadows it instead of leaking back out.
        ctx.provide(route.clone());
        matched.component.clone()
    }
}

/// A router mounted as the app's root.
///
/// Create it once, usually in [`xs_core::Backend`]'s initializer or wherever
/// the app builds its root element, and hand it to the event loop. The
/// argument is the root of one [`Route`] tree; the router itself implements
/// [`Render`], so it is the root:
///
/// ```ignore
/// Application::new().run(|_ctx| {
///     router(route("/", div()))
/// });
/// ```
///
/// One app mounts one router. Hooks such as [`crate::use_route`] and
/// [`crate::outlet`] read the [`RouteContext`] the router provides into the
/// render context before each route component runs, so they work anywhere in
/// that component's subtree.
#[derive(Clone)]
pub struct Router {
    inner: Rc<RouterInner>,
}

impl Router {
    /// Mounts a route tree on the platform's history.
    ///
    /// On wasm that is the browser's real history, so the address bar follows
    /// navigation and the back button works. Everywhere else it is an
    /// in-memory stack starting at `/`.
    pub fn new(root: Route) -> Self {
        Self::with_history(root, platform_history())
    }

    /// Mounts a route tree on a history the caller owns.
    ///
    /// Tests pass a [`crate::MemoryHistory`] with a known starting location;
    /// a native app with deep links can do the same with the path it was
    /// started with.
    pub fn with_history(root: Route, history: impl History + 'static) -> Self {
        let history: Box<dyn History> = Box::new(history);
        let location = State::new(history.location());
        let inner = Rc::new(RouterInner {
            history,
            location,
            root,
            matched: RefCell::new(Rc::new(Vec::new())),
        });

        let weak = Rc::downgrade(&inner);
        inner.history.listen(Rc::new(move |location| {
            if let Some(inner) = weak.upgrade() {
                inner.location.set(location);
            }
        }));

        Self { inner }
    }

    /// A handle that can navigate and read the location from anywhere.
    #[must_use]
    pub fn handle(&self) -> RouterHandle {
        RouterHandle {
            inner: Rc::clone(&self.inner),
        }
    }
}

/// Mounts a route tree on the platform's history.
///
/// Shorthand for [`Router::new`], so the root of an app reads as a call rather
/// than a constructor:
///
/// ```ignore
/// router(
///     route("/", layout())
///         .child(index(home()))
///         .child(route("users", users())),
/// )
/// ```
#[must_use]
pub fn router(root: Route) -> Router {
    Router::new(root)
}

impl Render for Router {
    fn id(&self) -> &'static str {
        "router"
    }

    fn render(&self, ctx: &mut Context) -> impl xs_core::IntoElement {
        let location = self.inner.location.read().clone();
        let matched = match_routes(&self.inner.root, &location.pathname);
        *self.inner.matched.borrow_mut() = Rc::new(matched.clone());

        let router = self.handle();
        let element = match matched.first() {
            Some(first) => self.inner.render_match(&router, ctx, first, 0, &location),
            None => empty(),
        };

        Rendered(element)
    }
}

/// A cheap clone of the router that can navigate and read the current
/// location, including outside rendering.
#[derive(Clone)]
pub struct RouterHandle {
    pub(crate) inner: Rc<RouterInner>,
}

impl RouterHandle {
    /// The current location.
    #[must_use]
    pub fn location(&self) -> Location {
        self.inner.location.read().clone()
    }

    /// Navigates to `to`, adding a history entry.
    pub fn push(&self, to: impl AsRef<str>) {
        self.inner.history.push(to.as_ref());
    }

    /// Navigates to `to`, replacing the current history entry.
    pub fn replace(&self, to: impl AsRef<str>) {
        self.inner.history.replace(to.as_ref());
    }

    /// Goes back one entry.
    pub fn back(&self) {
        self.inner.history.back();
    }

    /// Goes forward one entry.
    pub fn forward(&self) {
        self.inner.history.forward();
    }

    /// Moves `delta` entries.
    pub fn go(&self, delta: i32) {
        self.inner.history.go(delta);
    }

    /// Whether the current path is `to`.
    ///
    /// Only the path participates, not the query or the fragment, which is the
    /// usual definition of an active link. `to` is resolved against the current
    /// location first, so a relative link works.
    #[must_use]
    pub fn is_active(&self, to: &str) -> bool {
        let current = self.location();
        current.pathname == current.resolve(to).pathname
    }

    pub(crate) fn render_matched(&self, ctx: &mut Context, index: usize) -> AnyElement {
        let matched = self.inner.matched.borrow().clone();
        let location = self.location();
        match matched.get(index) {
            Some(matched) => self
                .inner
                .render_match(self, ctx, matched, index, &location),
            None => empty(),
        }
    }
}

#[cfg(test)]
mod tests {
    use xs_core::IntoElement;

    use super::{Router, router};
    use crate::{Location, MemoryHistory, index, layout, route};

    fn component() -> impl IntoElement {
        xs_primitive::div()
    }

    fn test_router() -> Router {
        Router::with_history(
            layout("")
                .child(route("/", component()))
                .child(route("/users/:id", component()))
                .child(
                    route("/dashboard", component())
                        .child(index(component()))
                        .child(route("settings", component())),
                ),
            MemoryHistory::new("/"),
        )
    }

    #[test]
    fn the_router_starts_at_the_history_location() {
        let router = Router::with_history(layout(""), MemoryHistory::new("/users/42"));
        assert_eq!(router.handle().location().pathname, "/users/42");
    }

    #[test]
    fn a_handle_navigates_the_router() {
        let router = test_router();
        let handle = router.handle();

        handle.push("/users/42");
        assert_eq!(handle.location().pathname, "/users/42");

        handle.back();
        assert_eq!(handle.location().pathname, "/");

        handle.forward();
        assert_eq!(handle.location().pathname, "/users/42");

        handle.replace("/dashboard");
        assert_eq!(handle.location().pathname, "/dashboard");

        handle.go(-1);
        assert_eq!(handle.location().pathname, "/");
    }

    #[test]
    fn is_active_ignores_query_and_fragment() {
        let router = Router::with_history(layout(""), MemoryHistory::new("/users/42?tab=x#top"));
        let handle = router.handle();
        assert!(handle.is_active("/users/42"));
        assert!(handle.is_active("/users/42?other=1"));
        assert!(!handle.is_active("/users"));
    }

    #[test]
    fn every_handle_sees_the_same_location() {
        let router = test_router();
        let first = router.handle();
        let second = router.handle();

        first.push("/dashboard/settings");
        assert_eq!(second.location(), Location::new("/dashboard/settings"));
    }

    #[test]
    fn the_router_helper_mounts_a_tree() {
        let mounted = router(layout("").child(route("/", component())));
        assert_eq!(mounted.handle().location().pathname, "/");
    }
}
