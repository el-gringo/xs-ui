use std::time::Duration;

use xs::{
    AlignItems, Context, Div, Easing, JustifyContent, ParentElement, Svg, WithAnimation, div, text,
};

use crate::theme;

static LOGO: Svg = Svg::new(include_str!("../../../ressources/images/xs-ui.svg"));

pub fn splash(ctx: &mut Context) -> Div {
    let height = (ctx.constraints.max.y * 0.22).clamp(96., 320.);

    div()
        .testid("splash")
        .column()
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .background(theme::BACK_950)
        .child(LOGO.vector(height * LOGO.aspect(), height))
        .with_animation(
            "spinner",
            Easing::new()
                .linear()
                .direction(xs::AnimationDirection::AlternateReverse)
                .infinite()
                .duration(Duration::from_secs(1)),
            |this, value| this.scale(1. + value),
        )
}

pub fn failed(error: &str) -> Div {
    div()
        .testid("failed")
        .column()
        .justify_content(JustifyContent::Center)
        .align_items(AlignItems::Center)
        .gap(12.)
        .background(theme::BACK_950)
        .child(
            div()
                .height(48.)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .font_size(48)
                .color(theme::NEUTRAL_300)
                .child(text("PeerTube")),
        )
        .child(
            div()
                .height(36.)
                .justify_content(JustifyContent::Center)
                .align_items(AlignItems::Center)
                .font_size(26)
                .color(theme::BRIGHT_RED)
                .child(text(error.to_owned())),
        )
}
