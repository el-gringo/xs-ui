use xs::{
    AlignItems, Color, Context, JustifyContent, NavigationNode, Overflow, ParentElement, Render,
    Svg, div, text,
};

use crate::{
    icons::ICON_SIZE,
    sidebar::{MARKER_GUTTER, MENU_WIDTH, RAIL_WIDTH},
    theme,
};

pub const BUTTON_HEIGHT: f32 = 50.;
const FOCUS_BORDER: f32 = 4.;

const ICON_GUTTER: f32 = (RAIL_WIDTH - MARKER_GUTTER - ICON_SIZE) / 2.;

const ICON_GAP: f32 = 24.;

const FONT_SIZE: u16 = 32;

pub struct Button {
    id: Option<&'static str>,
    icon: Option<&'static Svg>,
    label: String,
    slide: f32,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Button {
            id: None,
            icon: None,
            label: label.into(),
            slide: 0.,
        }
    }

    pub fn id(mut self, id: &'static str) -> Self {
        self.id = Some(id);
        self
    }

    pub fn icon(mut self, icon: &'static Svg) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn slide(mut self, slide: f32) -> Self {
        self.slide = slide;
        self
    }
}

impl Render for Button {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        let mut node = NavigationNode::new();
        if let Some(id) = self.id {
            node = node.id(id);
        }
        let nav_node = ctx.focus_handle(node);

        let (colour, border) = if nav_node.has_focus() {
            (theme::ACCENT_400, theme::ACCENT_400)
        } else if nav_node.is_selected() {
            (theme::ACCENT_400, Color::TRANSPARENT)
        } else {
            (theme::NEUTRAL_300, Color::TRANSPARENT)
        };

        div()
            .testid(self.id.unwrap_or("button"))
            .row()
            .justify_content(JustifyContent::FlexStart)
            .align_items(AlignItems::Center)
            .height(BUTTON_HEIGHT)
            .padding_left(ICON_GUTTER)
            .background_color(theme::BACK_900)
            .color(colour)
            .border_left(border, FOCUS_BORDER)
            .font_size(FONT_SIZE)
            .when(self.icon.is_some(), |this| {
                let icon = self.icon.expect("just asked");
                this.child(icon.vector_with(ICON_SIZE, ICON_SIZE, |image| image.tint(colour)))
            })
            .child(
                div()
                    .row()
                    .align_items(AlignItems::Center)
                    .height(BUTTON_HEIGHT)
                    .overflow(Overflow::Hidden)
                    .padding_left(ICON_GAP)
                    .color(colour)
                    .child(
                        div()
                            .align_items(AlignItems::Center)
                            .color(colour)
                            .translate_xy(-MENU_WIDTH * self.slide, 0.)
                            .child(text(self.label.clone())),
                    ),
            )
    }
}

pub fn button(label: impl Into<String>) -> Button {
    Button::new(label)
}
