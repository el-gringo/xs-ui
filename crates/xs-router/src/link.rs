use xs_core::{AnyElement, Context, IntoElement, ParentElement, Render};
use xs_primitive::{Div, div};

use crate::{
    RouterHandle,
    route::{Rendered, use_router},
};

/// A route-aware element that knows where it points.
///
/// The renderer has no pointer events yet, so a link is not clickable by
/// itself. Build it in a route component and call [`Link::activate`] from
/// whatever input the app does have - a focus handler, a key press, a test.
/// It is still the place where the target, the replace flag and the active
/// state live, so the UI does not have to repeat them.
///
/// ```ignore
/// let about = link(ctx, "/about").testid("about-link").child(text("About"));
/// ```
#[derive(Clone)]
pub struct Link {
    to: String,
    replace: bool,
    router: RouterHandle,
    testid: Option<&'static str>,
    div: Div,
}

/// A link to `to` on the router of the route currently rendering.
///
/// Panics outside a route component; use [`Link::new`] with an explicit handle
/// when there is no current route.
#[must_use]
#[track_caller]
pub fn link(ctx: &Context, to: impl Into<String>) -> Link {
    Link::new(to, use_router(ctx))
}

impl Link {
    /// A link to `to` on an explicit router.
    #[must_use]
    pub fn new(to: impl Into<String>, router: RouterHandle) -> Self {
        Self {
            to: to.into(),
            replace: false,
            router,
            testid: None,
            div: div(),
        }
    }

    /// Replaces the current history entry instead of pushing a new one.
    #[must_use]
    pub fn replace(mut self, replace: bool) -> Self {
        self.replace = replace;
        self
    }

    /// An id for tests and debug output.
    #[must_use]
    pub fn testid(mut self, testid: &'static str) -> Self {
        self.testid = Some(testid);
        self
    }

    /// The target, as written.
    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    /// Whether the current path is this link's target.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.router.is_active(&self.to)
    }

    /// Navigates to the target.
    pub fn activate(&self) {
        if self.replace {
            self.router.replace(&self.to);
        } else {
            self.router.push(&self.to);
        }
    }
}

impl ParentElement for Link {
    fn child(mut self, child: impl Into<AnyElement>) -> Self {
        self.div = self.div.child(child);
        self
    }

    fn children(mut self, children: impl IntoIterator<Item = impl Into<AnyElement>>) -> Self {
        self.div = self.div.children(children);
        self
    }
}

impl Render for Link {
    fn id(&self) -> &'static str {
        "link"
    }

    fn render(&self, _ctx: &mut Context) -> impl IntoElement {
        let mut div = self.div.clone();
        if let Some(testid) = self.testid {
            div = div.testid(testid);
        }
        Rendered(div.into_element())
    }
}

#[cfg(test)]
mod tests {
    use super::Link;
    use crate::{MemoryHistory, Route, Router};

    #[test]
    fn a_link_activates_the_router() {
        let router = Router::with_history(Route::layout(""), MemoryHistory::new("/"));
        let handle = router.handle();
        let link = Link::new("/users/42", handle.clone());

        assert!(!link.is_active());
        link.activate();
        assert_eq!(handle.location().pathname, "/users/42");
        assert!(link.is_active());
    }

    #[test]
    fn a_replacing_link_does_not_grow_the_stack() {
        let router = Router::with_history(Route::layout(""), MemoryHistory::new("/"));
        let handle = router.handle();
        handle.push("/users");
        let link = Link::new("/settings", handle.clone()).replace(true);

        link.activate();
        assert_eq!(handle.location().pathname, "/settings");
        handle.back();
        assert_eq!(
            handle.location().pathname,
            "/",
            "a push would have gone back to /users"
        );
    }
}
