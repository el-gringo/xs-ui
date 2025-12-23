#![allow(dead_code)]

use xs_backend::Backend;
use xs_layout::{
    Axis, Borders, Color, CrossAxisAlignment, EdgeInsets, Layout,
    MainAxisAlignment, colors::{BLUE, GREEN, RED, YELLOW},
};
use xs_primitives::{
    component, rsx, div,
    style::{Style, StyleDisplay},
};
mod app;

const WIDTH: f32 = 960.;
const HEIGH: f32 = 540.;

#[component]
pub fn div2() -> Box<dyn Layout> {
    let rootStyle = Style {
        width: Some(300.),
        background_color: Color::rgba(0.125, 0.125, 0.125, 255.),
        display: StyleDisplay::Flex,
        flex_direction: Axis::Column,
        justify_content: MainAxisAlignment::Start,
        align_items: CrossAxisAlignment::Center,
        ..Style::default()
    };
    let child1Style = Style {
        width: Some(200.),
        height: Some(40.),
        background_color: RED,
        margin: EdgeInsets {
            top: 8.,
            bottom: 0.,
            left: 16.,
            right: 16.,
        },
        border: Borders::all(YELLOW, 2.),
        ..Style::default()
    };
    let child2Style = Style {
        background_color: GREEN,
        ..child1Style
    };
    let child3Style = Style {
        background_color: BLUE,
        ..child1Style
    };

    rsx! {
        <div style={rootStyle}>
            <div style={child1Style}></div>
            <div style={child2Style}></div>
            <div style={child3Style}></div>
            </div>
        </div>
    }
}

pub fn main() {
    let mut backend = Backend::new(1280, 720);
    backend.run(|| div2::render(div2::Props {}));
}
