//! `translate`, `scale` and `rotate` on a `Div`.
//!
//! Each card below shows one setter. Everything is laid out normally and then
//! moved on the way to the screen, so the layout pass never learns about it and a
//! transformed element still takes up exactly the space it would without a
//! transform - siblings do not move out from under it.
//!
//! Rotation and scale happen about the element's own centre, the same default CSS
//! calls `transform-origin: 50% 50%`. Positive rotation is **clockwise**, because
//! this framework's screen space has y growing downwards.

use xs::{AlignItems, Application, Div, JustifyContent, prelude::*};

struct Transform;

impl Render for Transform {
    fn render(&self, ctx: &mut Context) -> impl IntoElement {
        let (screen_w, screen_h) = (ctx.app_context.screen.width, ctx.app_context.screen.height);

        div()
            .testid("root")
            .width(screen_w as _)
            .height(screen_h as _)
            .background_color(Color::rgba(0.08, 0.09, 0.11, 1.0))
            .row()
            .gap(40.)
            .padding(60.)
            .align_items(AlignItems::Center)
            .child(
                div()
                    .testid("translate")
                    .column()
                    .gap(12.)
                    .width(220.)
                    .child(card("translate", Color::BLUE))
                    .child(
                        // Pushed off its own row, not the row's.
                        div()
                            .translate_xy(60., 40.)
                            .height(40.)
                            .background(Color::BLUE),
                    ),
            )
            .child(
                div()
                    .testid("scale")
                    .column()
                    .gap(12.)
                    .width(220.)
                    .child(
                        card("scale", Color::GREEN)
                            // Grows about the centre, so it overhangs both sides.
                            .scale(1.3),
                    )
                    .child(card("scale_xy", Color::GREEN).scale_xy(1.6, 0.7)),
            )
            .child(
                div()
                    .testid("rotate")
                    .column()
                    .gap(12.)
                    .width(220.)
                    .child(
                        card("rotate", Color::RED)
                            // 15 degrees clockwise.
                            .with_animation(
                                "rotate+scale",
                                Easing::new().linear().repeat(u32::MAX),
                                |this, value| this.rotate(value),
                            ),
                    )
                    // Rotated and scaled together, to show they compose about the
                    // same centre rather than fighting over the origin.
                    .child(
                        card("rotate + scale", Color::RED).with_animation(
                            "rotate+scale",
                            Spring::new()
                                .direction(xs::AnimationDirection::AlternateReverse)
                                .repeat(u32::MAX),
                            |this, value| this.rotate(value).scale(value + 1.),
                        ),
                    ),
            )
            .child(
                div()
                    .testid("subtree")
                    .column()
                    .gap(12.)
                    .width(220.)
                    .child(
                        // The transform carries the whole subtree with it: this
                        // outer card is rotated, so its child sits at the same
                        // rotated angle without asking for it.
                        card_with_child("subtree", Color::rgba(0.9, 0.6, 0.1, 1.0)).with_animation(
                            "subtree",
                            Spring::new()
                                .direction(xs::AnimationDirection::AlternateReverse)
                                .repeat(u32::MAX),
                            |this, value| this.rotate(value),
                        ),
                    )
                    .child(
                        // Corner radii, borders and shadows stay round under this.
                        div()
                            .width(200.)
                            .height(120.)
                            .background(Color::rgba(0.5, 0.3, 0.8, 1.0))
                            .border_radius_all(24.)
                            .border(Color::WHITE, 4.)
                            .with_animation(
                                "cornet-radius-shadow",
                                Spring::new()
                                    .direction(xs::AnimationDirection::AlternateReverse)
                                    .repeat(u32::MAX),
                                |this, value| this.rotate(value),
                            ),
                    ),
            )
    }
}

/// One labelled card, centred in its own box so the rotation examples read
/// cleanly.
fn card(label: &str, color: Color) -> Div {
    div()
        .width(200.)
        .height(120.)
        .background(color)
        .border_radius_all(12.)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .color(Color::WHITE)
        .child(text(label))
}

/// A card with a child, for showing that a transform reaches the whole subtree.
fn card_with_child(label: &str, color: Color) -> Div {
    div()
        .width(200.)
        .height(120.)
        .background(color)
        .border_radius_all(12.)
        .align_items(AlignItems::Center)
        .justify_content(JustifyContent::Center)
        .color(Color::rgba(0.1, 0.1, 0.1, 1.0))
        .child(text(label))
        .child(
            div()
                .width(60.)
                .height(6.)
                .margin_top(8.)
                .border_radius_all(3.)
                .background(Color::rgba(0.1, 0.1, 0.1, 1.0)),
        )
}

fn main() {
    Application::new()
        .id("transform")
        .title("Transform")
        .run(|_ctx| Transform);
}
