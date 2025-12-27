use super::*;
use xs_layout::{View, colors::GRAY};
use xs_primitive::{
    component, div, rsx,
    style::{Style, StyleDisplay},
};


const WIDTH: f32 = 960.;
const HEIGH: f32 = 540.;

#[component]
pub fn sidebar(color: Color, children: Vec<View>) -> View {
    rsx! {
        <div style={Style {
            width: Some(300.),
            background_color: color,
            display: StyleDisplay::Flex,
            flex_direction: Axis::Column,
            justify_content: MainAxisAlignment::Start,
            align_items: CrossAxisAlignment::Center,
            ..Style::default()
        }}>
        {children}
        </div>
    }
}

#[component]
pub fn button(color: Color, children: Vec<View>) -> View {
    rsx! {
        <div style={Style {
            width: Some(200.),
            height: Some(40.),
            background_color: color,
            margin: EdgeInsets {
                top: 16.,
                bottom: 0.,
                left: 16.,
                right: 16.,
            },
            border: Borders::all(YELLOW, 2.),
            ..Style::default()
        }}>{children}</div>
    }
}

#[component]
pub fn div2() -> View {
    let rootStyle = Style {
        width: Some(300.),
        background_color: GRAY,
        display: StyleDisplay::Flex,
        flex_direction: Axis::Column,
        justify_content: MainAxisAlignment::Start,
        align_items: CrossAxisAlignment::Center,
        ..Style::default()
    };

    rsx! {
        <div style={rootStyle}>
            <button color={RED}></div>
            <button color={GREEN}></div>
            <button color={BLUE}></div>
        </div>
    }
}

pub fn main() {
    let mut backend = Backend::new(1280, 720);
    backend.run(|| div2::render(div2::Props {}));
}
