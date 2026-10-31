//! A small router demo.
//!
//! Left/Right moves between the nav links, Down enters the page, and Enter
//! follows the focused link or selects a user. On the web the address bar and
//! the browser's back button follow the router.

use xs::{
    AlignItems, Application, Context, Div, JustifyContent, KeyCode, NavigationNode, prelude::*,
    rgba,
};

const USERS: usize = 5;

/// The nav bar and footer are stated in pixels so the page below them gets an
/// explicit height; `div()` stretches to whatever box it is offered otherwise.
const NAV_HEIGHT: f32 = 80.0;
const FOOTER_HEIGHT: f32 = 44.0;

/// The card's outer width, and the width every row of content inside it uses.
const CARD_WIDTH: f32 = 640.0;
const CARD_CONTENT: f32 = 576.0;

const ROW_HEIGHT: f32 = 56.0;

fn main() {
    Application::new().title("xs-router").run(|_ctx| {
        router(
            route("/", layout())
                .child(index(home()))
                .child(route("users", users()))
                .child(route("users/:id", user()))
                .child(route("about", about()))
                .child(route("*", not_found())),
        )
    });
}

struct Layout;
impl Render for Layout {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let screen = ctx.app_context.screen;
        let _ = ctx.focus_handle(NavigationNode::vertical().id("app"));
        div()
            .column()
            .width(screen.width as f32)
            .height(screen.height as f32)
            .background_color(rgba(0.09, 0.10, 0.13, 1.0))
            .child(menu())
            .child(
                div()
                    .column()
                    .width(screen.width as f32)
                    .height(screen.height as f32 - NAV_HEIGHT - FOOTER_HEIGHT)
                    .align_items(AlignItems::Center)
                    .child(div().width(screen.width as f32).height(48.0))
                    .child(outlet(ctx)),
            )
            .child(
                div()
                    .width(screen.width as f32)
                    .height(FOOTER_HEIGHT)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .font_size(15)
                    .color(rgba(1.0, 1.0, 1.0, 0.35))
                    .child(text(
                        "Left/Right move  ·  Down enters a page  ·  Enter selects",
                    )),
            )
    }
}
fn layout() -> Layout {
    Layout
}

struct Menu;
impl Render for Menu {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let menu = ctx.focus_handle(NavigationNode::horizontal().id("menu"));
        let router = use_router(ctx);
        let width = ctx.app_context.screen.width as f32;
        div()
            .row()
            .width(width)
            .height(NAV_HEIGHT)
            .gap(8.0)
            .align_items(AlignItems::Center)
            .background_color(rgba(0.06, 0.07, 0.09, 1.0))
            .border_bottom(rgba(1.0, 1.0, 1.0, 0.07), 1.0)
            .child(div().width(32.0).height(NAV_HEIGHT - 1.0))
            .child(
                div()
                    .column()
                    .width(120.0)
                    .height(NAV_HEIGHT - 1.0)
                    .justify_content(JustifyContent::Center)
                    .font_size(18)
                    .color(rgba(1.0, 1.0, 1.0, 0.45))
                    .child(text("XS ROUTER")),
            )
            .child(div().grow(1.0, 1.0))
            .children(
                [("/", "Home"), ("/users", "Users"), ("/about", "About")]
                    .iter()
                    .map(|(path, label)| {
                        let active = router.is_active(path);
                        let router = router.clone();
                        // Back to the menu before every item: creating a focus
                        // node moves the cursor to it, and these three are
                        // siblings rather than a chain.
                        ctx.nav_id = menu.id;
                        button(ctx, path, label, active, move |key| {
                            if key == KeyCode::Enter {
                                router.push(path);
                                false
                            } else {
                                true
                            }
                        })
                    }),
            )
            .child(div().width(32.0).height(NAV_HEIGHT - 1.0))
    }
}

fn menu() -> Menu {
    Menu
}

fn button<H>(
    ctx: &mut Context,
    key: &'static str,
    label: &str,
    active: bool,
    handle_keydown: H,
) -> Div
where
    H: Fn(KeyCode) -> bool + 'static,
{
    // A distinct key per button: an unkeyed handle is keyed by call site,
    // which is the same for every item in the loop above.
    let mut handle = ctx.focus_handle_with_key(key, NavigationNode::leaf().id("menu-item"));
    handle.on_key_down(handle_keydown);
    div()
        .width(112.0)
        .height(44.0)
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .border_radius_all(10.0)
        .font_size(20)
        .color(if handle.has_focus() || active {
            Color::WHITE
        } else {
            rgba(1.0, 1.0, 1.0, 0.62)
        })
        .background_color(if handle.has_focus() {
            rgba(0.28, 0.47, 0.92, 1.0)
        } else if active {
            rgba(0.16, 0.20, 0.31, 1.0)
        } else {
            Color::TRANSPARENT
        })
        .child(text(label))
}

struct Page {
    title: String,
    body: String,
    children: Vec<AnyElement>,
}

impl ParentElement for Page {
    fn child(mut self, child: impl Into<AnyElement>) -> Self {
        self.children.push(child.into());
        self
    }

    fn children(mut self, children: impl IntoIterator<Item = impl Into<AnyElement>>) -> Self {
        self.children
            .extend(children.into_iter().map(|child| child.into()));
        self
    }
}

impl Render for Page {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let handle = ctx.focus_handle_with_key("page", NavigationNode::vertical().id("page"));
        let focused = handle.has_focus();
        div()
            .column()
            .width(CARD_WIDTH)
            .align_items(AlignItems::Center)
            .padding_top(32.0)
            .gap(14.0)
            .border_radius_all(18.0)
            .border(
                if focused {
                    rgba(0.36, 0.55, 1.0, 1.0)
                } else {
                    rgba(1.0, 1.0, 1.0, 0.08)
                },
                1.0,
            )
            .background_color(if focused {
                rgba(0.13, 0.16, 0.22, 1.0)
            } else {
                rgba(0.11, 0.12, 0.16, 1.0)
            })
            .box_shadow(0.0, 12.0, 32.0, 0.0, rgba(0.0, 0.0, 0.0, 0.28))
            .child(
                div().column().width(CARD_CONTENT).height(4.0).child(
                    div()
                        .width(44.0)
                        .height(4.0)
                        .border_radius_all(2.0)
                        .background_color(if focused {
                            rgba(0.42, 0.62, 1.0, 1.0)
                        } else {
                            rgba(0.28, 0.35, 0.55, 1.0)
                        }),
                ),
            )
            .child(
                div()
                    .column()
                    .width(CARD_CONTENT)
                    .height(48.0)
                    .justify_content(JustifyContent::Center)
                    .font_size(40)
                    .color(Color::WHITE)
                    .child(text(&self.title)),
            )
            .child(
                div()
                    .column()
                    .width(CARD_CONTENT)
                    .height(26.0)
                    .justify_content(JustifyContent::Center)
                    .font_size(24)
                    .color(rgba(1.0, 1.0, 1.0, 0.3))
                    .child(text(&self.body)),
            )
            .children(self.children.clone())
    }
}

fn page(title: impl Into<String>, body: impl Into<String>) -> Page {
    Page {
        title: title.into(),
        body: body.into(),
        children: Vec::new(),
    }
}

fn home() -> impl IntoElement {
    page("Home", "A small, keyboard-driven router demo.")
        .child(feature(
            "URL history",
            "The address bar and back button follow the router.",
        ))
        .child(feature(
            "Nested routes",
            "Pages render through the layout's outlet.",
        ))
        .child(feature("Route params", "User ids come from the path."))
}

struct UserRow {
    user_id: usize,
}

impl Render for UserRow {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        // The row was built by the users page, so the router's route context is
        // already in the render context; no lifetime to carry around.
        let route = use_route(ctx);
        let target = route.router().clone();
        let user_id = self.user_id;

        let mut handle = ctx.focus_handle(NavigationNode::leaf());
        handle.on_key_down(move |key| {
            if key == KeyCode::Enter {
                target.push(format!("/users/{user_id}"));
                false
            } else {
                true
            }
        });

        let active = handle.is_selected();
        let highlight = handle.has_focus();

        div()
            .row()
            .width(CARD_CONTENT - 12.0)
            .height(ROW_HEIGHT)
            .padding_left(12.0)
            .gap(12.0)
            .align_items(AlignItems::Center)
            .border_radius_all(10.0)
            .background_color(if highlight {
                rgba(0.22, 0.34, 0.62, 1.0)
            } else if active {
                rgba(0.15, 0.18, 0.26, 1.0)
            } else {
                rgba(0.12, 0.13, 0.17, 1.0)
            })
            .child(
                div()
                    .width(32.0)
                    .height(32.0)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .border_radius_all(16.0)
                    .font_size(15)
                    .color(Color::WHITE)
                    .background_color(if highlight {
                        rgba(0.36, 0.55, 1.0, 1.0)
                    } else {
                        rgba(0.27, 0.30, 0.38, 1.0)
                    })
                    .child(text(self.user_id.to_string())),
            )
            .child(
                div()
                    .column()
                    .width(220.0)
                    .height(52.0)
                    .child(
                        div()
                            .column()
                            .height(30.0)
                            .justify_content(JustifyContent::Center)
                            .font_size(26)
                            .color(if highlight {
                                Color::WHITE
                            } else {
                                rgba(1.0, 1.0, 1.0, 0.76)
                            })
                            .child(text(format!("User {}", self.user_id))),
                    )
                    .child(
                        div()
                            .column()
                            .height(22.0)
                            .justify_content(JustifyContent::Center)
                            .font_size(18)
                            .color(rgba(1.0, 1.0, 1.0, if highlight { 0.7 } else { 0.35 }))
                            .child(text("View profile")),
                    ),
            )
            .child(div().grow(1.0, 1.0))
            .child(
                div()
                    .width(32.0)
                    .height(ROW_HEIGHT)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .font_size(18)
                    .color(rgba(1.0, 1.0, 1.0, if highlight { 0.8 } else { 0.25 }))
                    .child(text("→")),
            )
    }
}

fn users() -> impl IntoElement {
    page("Users", "Choose a user with Up/Down, then press Enter.").child(
        div()
            .column()
            .width(CARD_CONTENT)
            .gap(8.0)
            .padding_top(8.0)
            .children((1..=USERS).map(|user_id| UserRow { user_id })),
    )
}

struct PageUser;
impl Render for PageUser {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let params = use_params(ctx);
        let id = params.param("id");
        let name = format!("User {id}");
        page(&name, "This user was selected from the list.")
            .child(
                div()
                    .row()
                    .width(CARD_CONTENT)
                    .height(72.0)
                    .gap(16.0)
                    .align_items(AlignItems::Center)
                    .child(
                        div()
                            .width(72.0)
                            .height(72.0)
                            .justify_content(JustifyContent::Center)
                            .align_items(AlignItems::Center)
                            .border_radius_all(36.0)
                            .font_size(30)
                            .color(Color::WHITE)
                            .background_color(rgba(0.28, 0.47, 0.92, 1.0))
                            .child(text(id)),
                    )
                    .child(
                        div()
                            .column()
                            .width(400.0)
                            .height(54.0)
                            .child(
                                div()
                                    .column()
                                    .height(32.0)
                                    .justify_content(JustifyContent::Center)
                                    .font_size(22)
                                    .color(Color::WHITE)
                                    .child(text(&name)),
                            )
                            .child(
                                div()
                                    .column()
                                    .height(22.0)
                                    .justify_content(JustifyContent::Center)
                                    .font_size(16)
                                    .color(rgba(1.0, 1.0, 1.0, 0.45))
                                    .child(text("Profile")),
                            ),
                    ),
            )
            .child(
                div()
                    .column()
                    .width(CARD_CONTENT)
                    .height(24.0)
                    .justify_content(JustifyContent::Center)
                    .font_size(16)
                    .color(rgba(1.0, 1.0, 1.0, 0.4))
                    .child(text("Use the browser's back button to return to the list.")),
            )
    }
}
fn user() -> PageUser {
    PageUser
}

fn about() -> impl IntoElement {
    page("About", "A react-router-shaped router for xs-ui.")
        .child(feature(
            "History",
            "pushState, replaceState and popstate on the web.",
        ))
        .child(feature(
            "Matching",
            "Static segments beat params, params beat splats.",
        ))
        .child(feature(
            "Composition",
            "Layouts render their match through an outlet.",
        ))
}

fn not_found() -> impl IntoElement {
    page("Not found", "That route does not exist.")
}

fn feature(title: &str, body: &str) -> Div {
    div()
        .column()
        .height(80.)
        .width(CARD_CONTENT)
        .gap(3.0)
        .child(
            div()
                .height(30.)
                .font_size(26)
                .color(rgba(1.0, 1.0, 1.0, 0.88))
                .child(text(title)),
        )
        .child(
            div()
                .font_size(28)
                .color(rgba(1.0, 1.0, 1.0, 0.45))
                .child(text(body)),
        )
}
