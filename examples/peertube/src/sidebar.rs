use std::time::Duration;

use xs::{
    AlignItems, Context, FocusHandle, JustifyContent, Overflow, ParentElement, Render, div, text,
};

use crate::{
    api::peertube::Category,
    button::button,
    icons::{self, for_category},
    theme,
};

pub const MENU_WIDTH: f32 = 300.;
pub const RAIL_WIDTH: f32 = 72.;

pub const MARKER_GUTTER: f32 = 20.;

pub const SLIDE: f32 = MENU_WIDTH - RAIL_WIDTH;

pub const COLLAPSE: Duration = Duration::from_millis(300);

const HEADER_HEIGHT: f32 = 75.;
const SIDEBAR_HEIGHT: f32 = 1080.;
const MENU_HEIGHT: f32 = SIDEBAR_HEIGHT - HEADER_HEIGHT;

const MENU_TOP: f32 = 64.;
const MENU_BOTTOM: f32 = 64.;
const MENU_GAP: f32 = 16.;

const VISIBLE_CATEGORIES: usize = 5;

pub struct Sidebar {
    categories: Vec<Category>,
    slide: f32,
    nav: FocusHandle,
}

impl Sidebar {
    pub fn categories(mut self, categories: Vec<Category>) -> Self {
        self.categories = categories.into_iter().take(VISIBLE_CATEGORIES).collect();
        self
    }

    pub fn slide(mut self, slide: f32) -> Self {
        self.slide = slide;
        self
    }
}

impl Render for Sidebar {
    fn render(&self, ctx: &mut Context) -> impl xs::IntoElement {
        ctx.nav_id = self.nav.id;

        div()
            .testid("sidebar")
            .column()
            .width(MENU_WIDTH)
            .justify_content(JustifyContent::FlexStart)
            .overflow(Overflow::Hidden)
            .background_color(theme::BACK_900)
            .translate_xy(-SLIDE * self.slide, 0.)
            .child(
                div()
                    .testid("sidebar-header")
                    .row()
                    .width(MENU_WIDTH)
                    .height(HEADER_HEIGHT)
                    .justify_content(JustifyContent::Center)
                    .align_items(AlignItems::Center)
                    .color(theme::NEUTRAL_300)
                    .font_size(40)
                    .child(text("PeerTube")),
            )
            .child(
                div()
                    .testid("sidebar-menu")
                    .column()
                    .align_items(AlignItems::FlexStart)
                    .width(MENU_WIDTH)
                    .height(MENU_HEIGHT)
                    .padding_left(MARKER_GUTTER)
                    .padding_top(MENU_TOP)
                    .padding_bottom(MENU_BOTTOM)
                    .gap(MENU_GAP)
                    .translate_xy(SLIDE * self.slide, 0.)
                    .children(self.categories.iter().map(|category| -> xs::AnyElement {
                        let mut item = button(category.label.clone()).slide(self.slide);
                        if let Some(icon) = for_category(category.id) {
                            item = item.icon(icon);
                        }
                        item.into()
                    }))
                    .child(div().grow(1., 1.))
                    .child(
                        button("Settings")
                            .icon(icons::settings())
                            .slide(self.slide)
                            .id("button-settings"),
                    ),
            )
    }
}

pub fn sidebar(nav: FocusHandle) -> Sidebar {
    Sidebar {
        categories: Vec::new(),
        slide: 0.,
        nav,
    }
}
