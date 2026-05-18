use std::{any::TypeId, rc::Rc};

use xs::{Application, Background, Color, Context, Element, NavigationNode, Render, div};

#[derive(Default)]
struct Row {
    top: f32,
    background: Background,
}

impl Row {
    pub fn top(mut self, top: f32) -> Self {
        self.top = top;
        self
    }

    pub fn background(mut self, background: impl Into<Background>) -> Self {
        self.background = background.into();
        self
    }
}

impl Render for Row {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }

    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let focus_handle = ctx.focus_handle(NavigationNode::new().vertical());
        let has_focus = focus_handle.has_focus();

        let border_color = if has_focus {
            Color::WHITE
        } else {
            Color::TRANSPARENT
        };
        div()
            .absolute()
            .grow(1.0, 0.)
            .height(100.)
            .top(self.top)
            .background(self.background)
            .padding_left(8.)
            .border_left(border_color, 8.)
            .into()
    }
}

impl From<Row> for Rc<dyn Render> {
    fn from(val: Row) -> Self {
        Rc::new(val)
    }
}

fn row() -> Row {
    Default::default()
}

struct App;
impl Render for App {
    fn tag(&self) -> std::any::TypeId {
        TypeId::of::<Self>()
    }
    fn render(&self, ctx: &mut Context) -> Box<dyn Element> {
        let _ = ctx.focus_handle(NavigationNode::new().vertical());
        div()
            .testid("root")
            .width(ctx.app_context.screen.width as _)
            .height(ctx.app_context.screen.height as _)
            .child(row().top(100.).background(Color::RED))
            .child(row().top(210.).background(Color::GREEN))
            .child(row().top(320.).background(Color::BLUE))
            .child(row().top(320.).background(Color::ORANGE))
            .child(row().top(320.).background(Color::YELLOW))
            .into()
    }
}

pub fn main() {
    xs_log::init();
    xs_log::info!("started !");
    Application::new()
        .id("playground")
        .title("Playground")
        .run(|_ctx| App);
}
