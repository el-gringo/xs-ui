use crate::{
    AlignContent, AlignItems, BorderRadius, BorderSide, Borders, BoxShadow, Color, EdgeInsets,
    FlexDirection, FlexWrap, JustifyContent,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient {
    pub start: Color,
    pub end: Color,
    pub angle: f32,
}

impl Gradient {
    #[must_use]
    pub fn linear(start: Color, end: Color) -> Self {
        Gradient {
            start,
            end,
            angle: 0.0,
        }
    }

    #[must_use]
    pub fn angled(start: Color, end: Color, angle: f32) -> Self {
        Gradient { start, end, angle }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Background {
    Color(Color),
    Gradient(Gradient),
}

impl Default for Background {
    fn default() -> Self {
        Background::Color(Color::TRANSPARENT)
    }
}

impl From<Color> for Background {
    fn from(color: Color) -> Self {
        Background::Color(color)
    }
}

impl From<Gradient> for Background {
    fn from(gradient: Gradient) -> Self {
        Background::Gradient(gradient)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct TextProperties {
    pub font_family: String,
    pub font_size: u16,
    pub font_weight: u16,
    pub font_style: FontStyle,
}

impl Default for TextProperties {
    fn default() -> Self {
        Self {
            font_family: String::from("sans-serif"),
            font_size: 16,
            font_weight: 400,
            font_style: FontStyle::Normal,
        }
    }
}

impl TextProperties {
    #[must_use]
    pub fn new(
        font_family: String,
        font_size: u16,
        font_weight: u16,
        font_style: FontStyle,
    ) -> Self {
        Self {
            font_family,
            font_size,
            font_weight,
            font_style,
        }
    }

    /// The same text at `scale` times the size.
    ///
    /// This is what keeps type sharp on a display with more than one device
    /// pixel per css pixel: the run is laid out and rasterised at the device
    /// size rather than being drawn at logical size and scaled up by the
    /// browser. The family, weight and style are not lengths and stay put.
    #[must_use]
    pub fn scaled(&self, scale: f32) -> TextProperties {
        TextProperties {
            font_size: (self.font_size as f32 * scale).round().max(1.0) as u16,
            ..self.clone()
        }
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum StyleDisplay {
    #[default]
    Flex,
    Block,
    Inline,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum StylePosition {
    #[default]
    Static,
    Absolute,
}

#[derive(Clone, Copy, Default, Debug, PartialEq)]
pub enum Overflow {
    #[default]
    Visible,
    Hidden,
}

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Style {
    /// Affine transform applied to this element and its whole subtree.
    ///
    /// Defaults to the identity, so an element without a transform lays out and
    /// draws exactly as it did before this field existed. Setters on the element
    /// (`translate`, `scale`, `rotate`) accumulate into it; see
    /// [`TransformOrigin`](crate::TransformOrigin) for where rotation happens
    /// about.
    pub transform: xs_math::Matrix3,

    pub color: Color,
    pub background: Background,
    pub background_image: Option<String>,

    pub border: Borders,
    pub border_top: Option<BorderSide>,
    pub border_right: Option<BorderSide>,
    pub border_bottom: Option<BorderSide>,
    pub border_left: Option<BorderSide>,

    pub border_radius: BorderRadius,
    pub border_top_left_radius: f32,
    pub border_top_right_radius: f32,
    pub border_bottom_right_radius: f32,
    pub border_bottom_left_radius: f32,

    pub box_shadow: BoxShadow,

    pub margin: EdgeInsets,
    pub margin_top: Option<f32>,
    pub margin_right: Option<f32>,
    pub margin_bottom: Option<f32>,
    pub margin_left: Option<f32>,

    pub padding: EdgeInsets,
    pub padding_top: Option<f32>,
    pub padding_right: Option<f32>,
    pub padding_bottom: Option<f32>,
    pub padding_left: Option<f32>,

    pub position: StylePosition,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub left: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub max_width: Option<f32>,
    pub max_height: Option<f32>,
    pub min_width: Option<f32>,
    pub min_height: Option<f32>,

    pub display: StyleDisplay,
    pub overflow: Overflow,

    // Flex properties
    pub flex_direction: FlexDirection,
    pub flex_wrap: FlexWrap,
    pub align_content: AlignContent,
    pub align_items: AlignItems,
    pub justify_content: JustifyContent,
    pub row_gap: f32,
    pub gap: f32,
    pub columns_gap: f32,
    pub order: u32,
    pub align_self: AlignItems,
    pub flex_basis: f32,
    pub flex_grow: [f32; 2],
    pub flex_shrink: f32,

    // Text properties
    pub text_properties: Option<TextProperties>,
}

impl Style {
    /// The same style at `scale` device pixels per logical pixel.
    ///
    /// Every length here is multiplied: the box, the insets, the border widths
    /// and radii, the shadow's offset, blur and spread, the gaps, the flex
    /// basis, and the type size. Everything that is not a length - colours,
    /// keywords, the discrete enums - is copied through, as are the factors
    /// rather than the offsets in [`Style::transform`], which
    /// [`xs_math::Matrix3::scale_translation`] knows the difference between.
    ///
    /// This is the one place logical pixels become device pixels. Doing it here
    /// rather than at the renderer means layout, the text atlas and the GL
    /// viewport all end up in the framebuffer's own units, so there is no
    /// second space to keep converting between. At `scale == 1.0` it is the
    /// identity, which is what a display with one device pixel per css pixel
    /// gets and what every unit test in the crate sees.
    #[must_use]
    pub fn scaled(&self, scale: f32) -> Style {
        let len = |v: Option<f32>| v.map(|v| v * scale);
        Style {
            transform: self.transform.scale_translation(scale),
            color: self.color,
            background: self.background,
            background_image: self.background_image.clone(),

            border: self.border.scaled(scale),
            border_top: self.border_top.map(|b| b.scaled(scale)),
            border_right: self.border_right.map(|b| b.scaled(scale)),
            border_bottom: self.border_bottom.map(|b| b.scaled(scale)),
            border_left: self.border_left.map(|b| b.scaled(scale)),

            border_radius: self.border_radius.scaled(scale),
            border_top_left_radius: self.border_top_left_radius * scale,
            border_top_right_radius: self.border_top_right_radius * scale,
            border_bottom_right_radius: self.border_bottom_right_radius * scale,
            border_bottom_left_radius: self.border_bottom_left_radius * scale,

            box_shadow: self.box_shadow.scaled(scale),

            margin: self.margin.scaled(scale),
            margin_top: len(self.margin_top),
            margin_right: len(self.margin_right),
            margin_bottom: len(self.margin_bottom),
            margin_left: len(self.margin_left),

            padding: self.padding.scaled(scale),
            padding_top: len(self.padding_top),
            padding_right: len(self.padding_right),
            padding_bottom: len(self.padding_bottom),
            padding_left: len(self.padding_left),

            position: self.position,
            top: len(self.top),
            right: len(self.right),
            bottom: len(self.bottom),
            left: len(self.left),
            width: len(self.width),
            height: len(self.height),
            max_width: len(self.max_width),
            max_height: len(self.max_height),
            min_width: len(self.min_width),
            min_height: len(self.min_height),

            display: self.display,
            overflow: self.overflow,

            // Flex properties
            flex_direction: self.flex_direction,
            flex_wrap: self.flex_wrap,
            align_content: self.align_content,
            align_items: self.align_items,
            justify_content: self.justify_content,
            row_gap: self.row_gap * scale,
            gap: self.gap * scale,
            columns_gap: self.columns_gap * scale,
            order: self.order,
            align_self: self.align_self,
            // A basis is a length; grow and shrink are ratios of the free space
            // and stay as they are.
            flex_basis: self.flex_basis * scale,
            flex_grow: self.flex_grow,
            flex_shrink: self.flex_shrink,

            text_properties: self
                .text_properties
                .as_ref()
                .map(|text_properties| text_properties.scaled(scale)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every length on a style, so a change that adds one without scaling it
    /// shows up here rather than on a HiDPI display.
    fn populated() -> Style {
        Style {
            color: Color::rgba(0.1, 0.2, 0.3, 0.4),
            background: Background::Gradient(Gradient::linear(
                Color::rgba(1., 0., 0., 1.),
                Color::rgba(0., 0., 1., 1.),
            )),
            border: Borders::all(Color::rgba(0., 0., 0., 1.), 2.),
            border_top: Some(BorderSide::new(Color::rgba(0., 0., 0., 1.), 3.)),
            border_radius: BorderRadius::new(1., 2., 3., 4.),
            box_shadow: BoxShadow::new(5., 6., 7., 8., Color::rgba(0., 0., 0., 1.)),
            margin: EdgeInsets::all(10.),
            padding: EdgeInsets::all(20.),
            top: Some(30.),
            width: Some(40.),
            gap: 50.,
            columns_gap: 60.,
            row_gap: 70.,
            flex_basis: 80.,
            flex_grow: [1., 2.],
            flex_shrink: 0.5,
            order: 3,
            text_properties: Some(TextProperties {
                font_family: String::from("serif"),
                font_size: 24,
                font_weight: 700,
                font_style: FontStyle::Italic,
            }),
            ..Default::default()
        }
    }

    #[test]
    fn scaling_by_one_changes_nothing() {
        // Every display at one device pixel per css pixel takes this path, and
        // it is also what every other test in the crate sees, so a non-identity
        // here would be a silent change to all of them.
        assert_eq!(populated().scaled(1.0), populated());
    }

    #[test]
    fn every_length_is_scaled() {
        let s = populated().scaled(2.0);
        assert_eq!(s.width, Some(80.));
        assert_eq!(s.top, Some(60.));
        assert_eq!(s.margin, EdgeInsets::all(20.));
        assert_eq!(s.padding, EdgeInsets::all(40.));
        assert_eq!(s.gap, 100.);
        assert_eq!(s.columns_gap, 120.);
        assert_eq!(s.row_gap, 140.);
        assert_eq!(s.flex_basis, 160.);
        assert_eq!(s.border.top.unwrap().width, 4.);
        assert_eq!(s.border_top.unwrap().width, 6.);
        assert_eq!(<[f32; 4]>::from(s.border_radius), [2., 4., 6., 8.]);
        assert_eq!(s.box_shadow.offset_left, 10.);
        assert_eq!(s.box_shadow.offset_top, 12.);
        assert_eq!(s.box_shadow.blur, 14.);
        assert_eq!(s.box_shadow.spread, 16.);
    }

    #[test]
    fn everything_that_is_not_a_length_is_left_alone() {
        let s = populated().scaled(3.0);
        let original = populated();
        assert_eq!(s.color, original.color);
        assert_eq!(s.background, original.background);
        assert_eq!(s.order, original.order);
        assert_eq!(s.flex_grow, original.flex_grow);
        assert_eq!(
            s.flex_shrink, original.flex_shrink,
            "a shrink factor is a ratio of the free space, not a length"
        );
    }

    #[test]
    fn the_type_size_is_scaled_and_the_face_is_not() {
        // The size is what makes the glyphs sharp; the family, weight and style
        // pick a different face and must not follow it.
        let s = populated().scaled(2.0);
        let text = s.text_properties.unwrap();
        assert_eq!(text.font_size, 48);
        assert_eq!(text.font_family, "serif");
        assert_eq!(text.font_weight, 700);
        assert_eq!(text.font_style, FontStyle::Italic);
    }

    #[test]
    fn a_transform_scales_its_offset_and_not_its_factors() {
        let style = Style {
            transform: xs_math::Matrix3::translation(xs_math::Vector2::new(10., 20.))
                * xs_math::Matrix3::uniform_scale(4.),
            ..Default::default()
        };
        let scaled = style.scaled(2.0);
        assert!(close(scaled.transform.scale_factor(), 4.0));
        assert!(close(
            scaled
                .transform
                .transform_point(xs_math::Vector2::new(0., 0.))
                .x,
            20.0
        ));
        assert!(close(
            scaled
                .transform
                .transform_point(xs_math::Vector2::new(0., 0.))
                .y,
            40.0
        ));
    }

    #[test]
    fn a_scaled_type_size_never_rounds_away_to_nothing() {
        // A scale of 0.01 on a 1px face is a rounding error, and a font size of
        // zero would divide by the atlas scale to make every glyph degenerate.
        let text = TextProperties {
            font_size: 1,
            ..Default::default()
        };
        assert_eq!(text.scaled(0.01).font_size, 1);
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1.0e-4
    }
}
