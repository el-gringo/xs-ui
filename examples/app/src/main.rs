use xs_backend::{AppContext, Backend, Element, NavigationNode, Render, focus::FocusHandle};
use xs_layout::{
    Color,
    colors::{BLUE, GRAY, GREEN, RED, WHITE},
};
use xs_primitive::div;

struct Sidebar {
    buttons: Vec<Button>,
}

impl Sidebar {
    pub fn new() -> Self {
        Sidebar {
            buttons: Vec::new(),
        }
    }

    #[track_caller]
    pub fn button(mut self, color: Color) -> Self {
        self.buttons.push(button(color));
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
            .width(300.)
            .column()
            .background_color(Color::rgba(0.128, 0.128, 0.128, 0.128))
            //.children(self.buttons)
            .child(Box::new(button(RED)))
            .child(Box::new(button(GREEN)))
            .child(Box::new(button(BLUE)))
            .into()
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
            .width(200.)
            .heigth(40.)
            .margin_top(16.)
            .margin_left(16.)
            .background_color(self.color)
            .border(if nav_node.has_focus() { WHITE } else { GRAY }, 2.)
            .into()
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

#[track_caller]
pub fn button(color: Color) -> Button {
    Button::new(color)
}

struct App;

impl Render for App {
    fn render(&self, _ctx: &mut AppContext) -> Box<dyn Element> {
        div()
            .child(sidebar().button(RED).button(GREEN).button(BLUE).into())
            .into()
    }
}

pub fn main() {
    let mut backend = Backend::new(1280, 720);
    backend.run(|| Box::new(App {}));
}
