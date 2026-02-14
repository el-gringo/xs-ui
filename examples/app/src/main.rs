#![allow(dead_code)]

use std::any::TypeId;

use xs::{
    AppContext, Application, BLUE, Color, Element, FocusHandle, GRAY, GREEN, NavigationNode, RED,
    Render, WHITE, div,
};

static BORDER_COLOR: Color = Color::rgba(0.27, 0.27, 0.27, 255.0);
static GREY1: Color = Color::rgba(0.09, 0.09, 0.09, 255.0);
static GREY2: Color = Color::rgba(0.11, 0.11, 0.11, 255.0);
static GREY3: Color = Color::rgba(0.16, 0.16, 0.16, 255.0);
static GREY4: Color = Color::rgba(0.24, 0.24, 0.24, 255.0);
static IMAGE1: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/fabcf1f1-5ef0-4551-9a3b-a6ff953bbcd9.jpg";
static IMAGE2: &str = "https://video.monsieurbidouille.fr/lazy-static/thumbnails/440cf8a3-1389-4475-9b65-6e1dba693235.jpg";

struct Sidebar {
    buttons: Vec<Button>,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            buttons: Vec::new(),
        }
    }

    pub fn button(mut self, button: Button) -> Self {
        self.buttons.push(button);
        self
    }

    pub fn buttons(mut self, buttons: Vec<Button>) -> Self {
        self.buttons.extend(buttons);
        self
    }
}

impl Render for Sidebar {
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element> {
        let _nav_node = ctx.use_state::<FocusHandle>(|ctx| {
            let node = NavigationNode::new("sidebar").vertical();
            ctx.nav.create_handle(node)
        });

        div()
            .id("sidebar")
            .width(300.)
            .column()
            .align_items(xs::CrossAxisAlignment::Center)
            .background_color(GREY2)
            .child(
                div()
                    .id("sidebar-header")
                    .width(300.)
                    .height(50.)
                    .background_color(GREY3)
                    .margin_bottom(32.)
                    .into(),
            )
            .child(Box::new(button(RED)))
            .child(Box::new(button(GREEN)))
            .child(Box::new(button(BLUE)))
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Sidebar>()
    }
}

pub struct Button {
    color: Color,
}

impl Button {
    pub fn new(color: Color) -> Self {
        Button { color }
    }
}

impl Render for Button {
    fn render(&self, ctx: &mut AppContext) -> Box<dyn Element> {
        let nav_node = ctx
            .use_state::<FocusHandle>(|ctx| ctx.nav.create_handle(NavigationNode::new("button")));
        div()
            .id("button")
            .width(200.)
            .height(40.)
            .margin_top(16.)
            .background_color(self.color)
            .border(if nav_node.has_focus() { WHITE } else { GRAY }, 2.)
            .into()
    }
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Button>()
    }
}

impl Into<Box<dyn Render>> for Sidebar {
    fn into(self) -> Box<dyn Render> {
        Box::new(self)
    }
}

fn sidebar() -> Sidebar {
    Sidebar::new()
}

pub fn button(color: Color) -> Button {
    Button::new(color)
}

struct MyApp;
impl Render for MyApp {
    fn render(&self, _ctx: &mut AppContext) -> Box<dyn Element> {
        div()
            .id("app")
            .row()
            .child(
                sidebar()
                    .buttons([button(RED), button(GREEN), button(BLUE)].into())
                    .into(),
            )
            .child(
                div()
                    .id("body")
                    .column()
                    .background_color(GREY3)
                    .child(
                        div()
                            .row()
                            //.justify_content(xs::MainAxisAlignment::Center)
                            .child(
                                div()
                                    .id("thumbnail1")
                                    .width(223.)
                                    .height(122.)
                                    .margin_right(32.)
                                    .background_color(GREY4)
                                    .background_image(IMAGE1)
                                    .into()
                            )
                            .child(
                                div()
                                    .id("thumbnail2")
                                    .width(223.)
                                    .height(122.)
                                    .background_color(GREY4)
                                    .background_image(IMAGE2)
                                    .into()
                            )
                            .into()
                    )
                    .into(),
            )
            .into()
    }

    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<MyApp>()
    }
}

pub fn main() {
    Application::new()
        .id("app")
        .title("App")
        .width(960)
        .height(540)
        .run(Box::new(MyApp {}));
}
