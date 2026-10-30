use std::rc::Rc;

use xs_core::{AnyElement, Context, IntoElement, Render};
use xs_primitive::div;

use crate::{Location, Params, RouterHandle, pattern::Pattern};

/// Renders one route.
///
/// A route component is a zero-argument function, and its result can be
/// anything that resolves to an [`AnyElement`] - a plain element, a
/// [`Render`] type, or an already-built [`AnyElement`]:
///
/// ```ignore
/// fn about() -> impl IntoElement {
///     div().child(text("About"))
/// }
///
/// struct User;
/// impl Render for User {
///     fn render(&self, ctx: &mut Context) -> impl IntoElement {
///         let id = use_params(ctx).param("id");
///         div().child(text(id))
///     }
/// }
///
/// fn user() -> User {
///     User
/// }
/// ```
///
/// A zero-argument function has no context of its own, so anything that needs
/// one - an outlet, a hook - lives in the element it returns, usually a
/// [`Render`] type as above.
pub trait RouteComponent: 'static {
    fn render(&self) -> AnyElement;
}

impl<F, E> RouteComponent for F
where
    F: Fn() -> E + 'static,
    E: Into<AnyElement> + 'static,
{
    fn render(&self) -> AnyElement {
        (self)().into()
    }
}

impl RouteComponent for AnyElement {
    fn render(&self) -> AnyElement {
        self.clone()
    }
}

impl RouteComponent for Rc<dyn RouteComponent> {
    fn render(&self) -> AnyElement {
        (**self).render()
    }
}

/// Everything a route component knows about the match it is rendering.
#[derive(Clone)]
pub struct RouteContext {
    pub(crate) router: RouterHandle,
    pub(crate) params: Params,
    pub(crate) location: Location,
    pub(crate) depth: usize,
    pub(crate) path: String,
}

impl RouteContext {
    /// The router, for navigating.
    #[must_use]
    pub fn router(&self) -> &RouterHandle {
        &self.router
    }

    /// The dynamic segments visible at this point in the tree.
    ///
    /// Parent parameters are merged in; a child wins on a collision.
    #[must_use]
    pub fn params(&self) -> &Params {
        &self.params
    }

    /// The location being rendered.
    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// How deep this route is. The root route is depth 0.
    #[must_use]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// The route's full pattern, e.g. `/users/:id`.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }
}

/// One node in the router's route tree.
///
/// A route has a path relative to its parent, a component, and optionally
/// children. An index route matches its parent exactly and is what makes a
/// layout route render something at its own path.
#[derive(Clone)]
pub struct Route {
    path: String,
    pattern: Pattern,
    index: bool,
    component: Option<AnyElement>,
    children: Vec<Route>,
}

impl Route {
    /// A route with a path and a component.
    pub fn new(path: impl Into<String>, component: impl Into<AnyElement>) -> Self {
        let path = path.into();
        Self {
            pattern: Pattern::new(&path),
            path,
            index: false,
            component: Some(component.into()),
            children: Vec::new(),
        }
    }

    /// A route that matches its parent's path exactly.
    pub fn index(component: impl Into<AnyElement>) -> Self {
        Self {
            path: String::new(),
            pattern: Pattern::new(""),
            index: true,
            component: Some(component.into()),
            children: Vec::new(),
        }
    }

    /// A route with a path and children but no component of its own.
    ///
    /// This is the pathless-or-pathful wrapper that gives a subtree a shared
    /// prefix without drawing anything itself.
    pub fn layout(path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            pattern: Pattern::new(&path),
            path,
            index: false,
            component: None,
            children: Vec::new(),
        }
    }

    /// Adds a child route. Its path is relative to this route's path.
    #[must_use]
    pub fn child(mut self, child: Self) -> Self {
        self.children.push(child);
        self
    }

    /// Adds several child routes.
    #[must_use]
    pub fn children(mut self, children: impl IntoIterator<Item = Self>) -> Self {
        self.children.extend(children);
        self
    }

    pub(crate) fn path(&self) -> &str {
        &self.path
    }

    pub(crate) fn pattern(&self) -> &Pattern {
        &self.pattern
    }

    pub(crate) fn is_index(&self) -> bool {
        self.index
    }

    pub(crate) fn component(&self) -> Option<&AnyElement> {
        self.component.as_ref()
    }

    pub(crate) fn child_routes(&self) -> &[Self] {
        &self.children
    }
}

/// A route with a path and a component.
///
/// This is [`Route::new`] in function form, so a route tree reads as a tree of
/// calls instead of a chain of constructors:
///
/// ```ignore
/// router(
///     route("/", layout)
///         .child(index(home))
///         .child(route("users", users)),
/// )
/// ```
#[must_use]
pub fn route(path: impl Into<String>, component: impl Into<AnyElement>) -> Route {
    Route::new(path, component)
}

/// A route that matches its parent's path exactly.
///
/// Shorthand for [`Route::index`].
#[must_use]
pub fn index(component: impl Into<AnyElement>) -> Route {
    Route::index(component)
}

/// A shared path prefix with no component of its own.
///
/// Shorthand for [`Route::layout`]. It draws nothing, so its children have to
/// match on their own; a component route that also renders an outlet is the
/// usual alternative.
#[must_use]
pub fn layout(path: impl Into<String>) -> Route {
    Route::layout(path)
}

/// The route currently being rendered.
///
/// The router provides the [`RouteContext`] into the render context before it
/// invokes a route component, so this reads it from anywhere in that
/// component's subtree - including a [`Render`] impl that never receives the
/// context argument.
///
/// Panics outside a route component.
#[must_use]
#[track_caller]
pub fn use_route(ctx: &Context) -> RouteContext {
    (*ctx.expect_context::<RouteContext>()).clone()
}

/// The router for the route currently being rendered.
#[must_use]
#[track_caller]
pub fn use_router(ctx: &Context) -> RouterHandle {
    use_route(ctx).router
}

/// The parameters of the route currently being rendered.
#[must_use]
#[track_caller]
pub fn use_params(ctx: &Context) -> Params {
    use_route(ctx).params
}

/// The location of the route currently being rendered.
#[must_use]
#[track_caller]
pub fn use_location(ctx: &Context) -> Location {
    use_route(ctx).location
}

/// Renders the child route of a route component.
///
/// Use it inside a route component that has children. The component needs the
/// render context to find the outlet, so it is a [`Render`] type rather than a
/// zero-argument closure:
///
/// ```ignore
/// struct Users;
/// impl Render for Users {
///     fn render(&self, ctx: &mut Context) -> impl IntoElement {
///         div().child(outlet(ctx))
///     }
/// }
///
/// route("/users", users)
///     .child(index(all_users))
///     .child(route(":id", user))
/// ```
///
/// [`crate::Route::layout`] has no component, so there is no component to put
/// the outlet in: it only gives its children a shared path prefix.
#[derive(Clone)]
pub struct Outlet {
    router: RouterHandle,
    next: usize,
}

/// The next route in the match chain, ready to be a child.
///
/// Panics when there is no current route, because an outlet only means
/// something inside a route component.
#[must_use]
#[track_caller]
pub fn outlet(ctx: &Context) -> Outlet {
    let current = use_route(ctx);
    Outlet {
        router: current.router,
        next: current.depth + 1,
    }
}

impl Render for Outlet {
    fn id(&self) -> &'static str {
        "outlet"
    }

    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        self.router.render_matched(ctx, self.next)
    }
}

/// An element that is already resolved.
///
/// `Render::render` has to return `impl IntoElement`, and `AnyElement` itself
/// is only the input to `IntoElement`, so route rendering wraps its result in
/// this.
pub(crate) struct Rendered(pub(crate) AnyElement);

impl IntoElement for Rendered {
    fn into_element(self) -> AnyElement {
        self.0
    }
}

/// The empty element a route renders when there is no child to show.
pub(crate) fn empty() -> impl IntoElement {
    div()
}

#[cfg(test)]
mod tests {
    use xs_core::{
        AnyElement, AppContext, BoxConstraints, Color, Context, IntoElement, TextProperties,
        TextSystemDummy,
    };

    use super::{Route, RouteContext, index, layout, route, use_route};
    use crate::{Location, Params};

    fn component() -> xs_primitive::Div {
        xs_primitive::div()
    }

    fn any_component() -> AnyElement {
        component().into_element()
    }

    fn with_context<R>(f: impl FnOnce(&mut Context) -> R) -> R {
        let mut app_context = AppContext::new(Box::new(TextSystemDummy::default()));
        let mut ctx = Context::new(
            &mut app_context,
            (std::any::TypeId::of::<u8>(), 0, 0),
            0,
            TextProperties::default(),
            Color::rgba(0., 0., 0., 1.),
            BoxConstraints::from_max((100., 100.)),
        );
        f(&mut ctx)
    }

    fn route_context(depth: usize, path: &str) -> RouteContext {
        RouteContext {
            router: crate::Router::new(Route::layout("")).handle(),
            params: Params::new(),
            location: Location::new("/"),
            depth,
            path: path.to_string(),
        }
    }

    #[test]
    fn a_route_tree_keeps_children_in_order() {
        let root = layout("").child(route("/", component)).child(
            layout("/users")
                .child(index(component))
                .child(route(":id", component)),
        );

        assert_eq!(root.child_routes().len(), 2);
        assert_eq!(root.child_routes()[1].child_routes().len(), 2);
    }

    #[test]
    fn a_route_component_can_return_an_any_element() {
        let root = route("/", any_component).child(route("users", any_component()));

        assert_eq!(root.child_routes().len(), 1);
        assert!(root.component().is_some());
        assert!(root.child_routes()[0].component().is_some());
    }

    #[test]
    fn a_route_is_read_from_the_render_context() {
        with_context(|ctx| {
            ctx.provide(route_context(3, "/users/:id"));
            assert_eq!(use_route(ctx).depth(), 3);
            assert_eq!(use_route(ctx).path(), "/users/:id");

            // The innermost scope wins, which is what lets a child route
            // shadow its parent's parameters and location.
            ctx.provide(route_context(4, "/users/:id/posts"));
            assert_eq!(use_route(ctx).path(), "/users/:id/posts");
        });
    }

    #[test]
    #[should_panic(expected = "no context of type")]
    fn reading_a_route_outside_a_route_component_panics() {
        with_context(|ctx| {
            let _ = use_route(ctx);
        });
    }
}
