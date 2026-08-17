use std::{any::TypeId, rc::Rc};

use xs_core::{
    AnimationState, AnyElement, AppContext, Context, ElementObject, IntoElement, ParentElement,
    TextProperties,
};
use xs_layout::{
    AlignItems, BorderRadius, BorderSide, Borders, BoxShadow, Color, EdgeInsets, Flex,
    FlexDirection, FlexWrap, JustifyContent, Material, Overflow, Positioned, QuadProps, Style,
    StyleDisplay, StylePosition, View,
};

pub struct DivAnimation {
    pub id: &'static str,
    pub animate: Box<dyn xs_animation::Animate>,
    pub callback: Rc<dyn Fn(Div, f32) -> Div>,
}

impl Clone for DivAnimation {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            animate: self.animate.clone_box(),
            callback: Rc::clone(&self.callback),
        }
    }
}

#[derive(Default, Clone)]
pub struct Div {
    pub testid: Option<&'static str>,
    pub style: Style,
    pub children: Vec<AnyElement>,
    pub animation: Option<DivAnimation>,
}

impl Div {
    pub fn new() -> Self {
        Default::default()
    }

    /// An id that is only used for debug and tests
    pub fn testid(mut self, testid: &'static str) -> Self {
        self.testid = Some(testid);
        self
    }

    //
    // POSITION
    //
    pub fn absolute(mut self) -> Self {
        self.style.position = StylePosition::Absolute;
        self
    }

    pub fn top(mut self, value: f32) -> Self {
        self.style.top = Some(value);
        self
    }

    pub fn right(mut self, value: f32) -> Self {
        self.style.right = Some(value);
        self
    }

    pub fn bottom(mut self, value: f32) -> Self {
        self.style.bottom = Some(value);
        self
    }

    pub fn left(mut self, value: f32) -> Self {
        self.style.left = Some(value);
        self
    }

    pub fn width(mut self, size: f32) -> Self {
        self.style.width = Some(size);
        self
    }

    pub fn height(mut self, size: f32) -> Self {
        self.style.height = Some(size);
        self
    }

    //
    // MARGIN
    //
    pub fn margin(mut self, margin: f32) -> Self {
        self.style.margin = EdgeInsets::all(margin);
        self
    }

    pub fn margin_top(mut self, size: f32) -> Self {
        self.style.margin.top = size;
        self
    }

    pub fn margin_right(mut self, size: f32) -> Self {
        self.style.margin.right = size;
        self
    }

    pub fn margin_bottom(mut self, size: f32) -> Self {
        self.style.margin.bottom = size;
        self
    }

    pub fn margin_left(mut self, size: f32) -> Self {
        self.style.margin.left = size;
        self
    }

    //
    // PADDING
    //
    pub fn padding(mut self, size: f32) -> Self {
        self.style.padding = EdgeInsets::all(size);
        self
    }

    pub fn padding_top(mut self, size: f32) -> Self {
        self.style.padding.top = size;
        self
    }

    pub fn padding_right(mut self, size: f32) -> Self {
        self.style.padding.right = size;
        self
    }

    pub fn padding_bottom(mut self, size: f32) -> Self {
        self.style.padding.bottom = size;
        self
    }

    pub fn padding_left(mut self, size: f32) -> Self {
        self.style.padding.left = size;
        self
    }

    //
    // BORDER
    //
    pub fn border(mut self, color: Color, width: f32) -> Self {
        self.style.border = Borders::all(color, width);
        self
    }

    pub fn border_top(mut self, color: Color, width: f32) -> Self {
        self.style.border.top = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_right(mut self, color: Color, width: f32) -> Self {
        self.style.border.right = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_bottom(mut self, color: Color, width: f32) -> Self {
        self.style.border.bottom = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_left(mut self, color: Color, width: f32) -> Self {
        self.style.border.left = Some(BorderSide::new(color, width));
        self
    }

    pub fn border_radius(
        mut self,
        top_left: f32,
        top_right: f32,
        bottom_right: f32,
        bottom_left: f32,
    ) -> Self {
        self.style.border_radius =
            BorderRadius::new(top_left, top_right, bottom_right, bottom_left);
        self
    }

    pub fn border_radius_all(mut self, radius: f32) -> Self {
        self.style.border_radius = BorderRadius::new(radius, radius, radius, radius);
        self
    }

    pub fn border_top_left_radius(mut self, radius: f32) -> Self {
        self.style.border_top_left_radius = radius;
        self
    }

    pub fn border_top_right_radius(mut self, radius: f32) -> Self {
        self.style.border_top_right_radius = radius;
        self
    }

    pub fn border_bottom_right_radius(mut self, radius: f32) -> Self {
        self.style.border_bottom_right_radius = radius;
        self
    }

    pub fn border_bottom_left_radius(mut self, radius: f32) -> Self {
        self.style.border_bottom_left_radius = radius;
        self
    }

    //
    // BOX SHADOW
    //
    pub fn box_shadow(
        mut self,
        offset_left: f32,
        offset_top: f32,
        blur: f32,
        spread: f32,
        color: Color,
    ) -> Self {
        self.style.box_shadow = BoxShadow::new(offset_left, offset_top, blur, spread, color);
        self
    }

    //
    // COLOR
    //
    pub fn color(mut self, color: impl Into<Color>) -> Self {
        self.style.color = color.into();
        self
    }

    pub fn background(mut self, bg: impl Into<xs_layout::Background>) -> Self {
        self.style.background = bg.into();
        self
    }

    pub fn background_color(mut self, color: Color) -> Self {
        self.style.background = xs_layout::Background::Color(color);
        self
    }

    pub fn background_image(mut self, image: String) -> Self {
        self.style.background_image = Some(image);
        self
    }

    //
    // FLEX
    //
    pub fn row(mut self) -> Self {
        self.style.display = StyleDisplay::Flex;
        self.style.flex_direction = FlexDirection::Row;
        self
    }

    pub fn column(mut self) -> Self {
        self.style.display = StyleDisplay::Flex;
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    pub fn justify_content(mut self, value: JustifyContent) -> Self {
        self.style.justify_content = value;
        self
    }

    pub fn align_items(mut self, value: AlignItems) -> Self {
        self.style.align_items = value;
        self
    }

    pub fn grow(mut self, main: f32, cross: f32) -> Self {
        self.style.flex_grow = [main, cross];
        self
    }

    pub fn shrink(mut self, shrink: f32) -> Self {
        self.style.flex_shrink = shrink;
        self
    }

    pub fn gap(mut self, gap: f32) -> Self {
        self.style.gap = gap;
        self
    }

    pub fn wrap(mut self) -> Self {
        self.style.flex_wrap = FlexWrap::Wrap;
        self
    }

    pub fn overflow(mut self, overflow: Overflow) -> Self {
        self.style.overflow = overflow;
        self
    }

    pub fn font_family(mut self, font_family: impl Into<String>) -> Self {
        self.update_text_properties(|text_properties| {
            text_properties.font_family = font_family.into()
        });
        self
    }

    pub fn font_size(mut self, font_size: u16) -> Self {
        self.update_text_properties(|text_properties| text_properties.font_size = font_size);
        self
    }

    pub fn font_weight(mut self, font_weight: u16) -> Self {
        self.update_text_properties(|text_properties| text_properties.font_weight = font_weight);
        self
    }

    fn update_text_properties(&mut self, mutator: impl FnOnce(&mut TextProperties)) {
        match self.style.text_properties.as_mut() {
            Some(v) => mutator(v),
            None => {
                let v = Default::default();
                self.style.text_properties.replace(v);
                self.update_text_properties(mutator);
            }
        }
    }

    pub fn when(self, condition: bool, mut callback: impl FnMut(Self) -> Self) -> Self {
        if condition { callback(self) } else { self }
    }
}

impl IntoElement for Div {
    fn into_element(self) -> AnyElement {
        AnyElement::new(self)
    }
}

impl ElementObject for Div {
    fn tag(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    fn clone_box(&self) -> Box<dyn ElementObject> {
        Box::new(self.clone())
    }

    fn id(&self) -> &'static str {
        self.testid.unwrap_or("div")
    }

    fn children(&self) -> &[AnyElement] {
        &self.children
    }

    fn post_render(&mut self, ctx: &mut Context) {
        if let Some(text_properties) = self.style.text_properties.clone() {
            ctx.text_properties = text_properties;
        }
        ctx.color = self.style.color;
        if let Some(w) = self.style.width {
            ctx.constraints.max.x = w;
        };
        if let Some(h) = self.style.height {
            ctx.constraints.max.y = h;
        };
        if let Some(ref anim) = self.animation {
            let now = ctx.app_context.now;
            let initial = anim.animate.clone_box();
            let incoming = anim.animate.clone_box();
            let callback = anim.callback.clone();

            let state = ctx.use_keyed_state::<AnimationState>(anim.id, move |_, _| {
                AnimationState::new_boxed(initial, now)
            });
            state.mutate(|s| s.retrigger_if_changed(incoming, now));
            let value = {
                let mut value = 0.0;
                state.mutate(|s| value = s.tick(now));
                value
            };
            if !state.read().is_finished() {
                ctx.app_context.dirty = true;
            }
            let this = (callback)(self.clone(), value);
            self.style = this.style;
        }
    }

    fn paint(&self, app_context: &mut AppContext, children: &[View]) -> View {
        let mut style = self.style.clone();
        let material = Material::Quad(Rc::new(QuadProps {
            background: style.background,
            border: style.border,
            border_radius: style.border_radius,
            box_shadow: style.box_shadow,
            overflow: style.overflow,
            background_image: app_context.load_image(style.background_image.clone()),
        }));
        match style.position {
            StylePosition::Absolute => {
                style.position = StylePosition::Static;
                let material = Some(material);
                Rc::new(Positioned {
                    position: (style.left.unwrap_or(0.), style.top.unwrap_or(0.)).into(),
                    child: Rc::new(Flex {
                        style,
                        material,
                        children: children.to_vec(),
                    }),
                })
            }
            StylePosition::Static => {
                let material = Some(material);
                Rc::new(Flex {
                    style,
                    material,
                    children: children.to_vec(),
                })
            }
        }
    }
}

impl ParentElement for Div {
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

pub fn div() -> Div {
    Div::new()
}
