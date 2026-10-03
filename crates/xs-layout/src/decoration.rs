use std::rc::Rc;

use crate::{Background, Color, Image, TextProperties, TextVertices};
use xs_math::Vector2;

#[derive(Clone, PartialEq, Debug)]
pub struct QuadProps {
    pub background: Background,
    pub border: Borders,
    pub border_radius: BorderRadius,
    pub box_shadow: BoxShadow,
    pub overflow: crate::Overflow,
    pub background_image: Option<Image>,
}

impl QuadProps {
    /// Returns true if this quad is a plain solid-color rectangle with no
    /// border, shadow, gradient, or border-radius. These can be rendered
    /// with a much simpler/faster shader path. Background images are
    /// allowed — the simple shader samples the texture and blends it
    /// with the background color.
    #[must_use]
    pub fn is_simple(&self) -> bool {
        matches!(self.background, Background::Color(_))
            && self.border == Borders::none()
            && self.border_radius == BorderRadius::default()
            && self.box_shadow == BoxShadow::default()
            && self.overflow == crate::Overflow::default()
    }

    /// If this is a simple quad, return the background color and optional
    /// texture ID (0 when there is no background image).
    #[must_use]
    pub fn simple_material(&self) -> Option<(Color, u32)> {
        if self.is_simple() {
            if let Background::Color(c) = self.background {
                let tex = self.background_image.as_ref().map_or(0, |img| img.texture);
                Some((c, tex))
            } else {
                unreachable!()
            }
        } else {
            None
        }
    }
}

impl Default for QuadProps {
    fn default() -> Self {
        Self {
            background: Background::Color(Color::TRANSPARENT),
            border: Borders::none(),
            border_radius: BorderRadius::default(),
            box_shadow: BoxShadow::default(),
            overflow: crate::Overflow::default(),
            background_image: None,
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Material {
    Quad(Rc<QuadProps>),
    /// Fast-path for plain rectangles: no border, no shadow, no gradient,
    /// no border-radius. Rendered with a dedicated lightweight shader.
    /// The u32 is the GL texture ID (0 = use the placeholder solid-color
    /// texture).
    SimpleQuad(Color, u32),
    Text(Color, TextProperties, Rc<TextVertices>),
}

impl Material {
    #[must_use]
    pub fn filled(background: impl Into<Background>) -> Material {
        Material::Quad(Rc::new(QuadProps {
            background: background.into(),
            ..Default::default()
        }))
    }
}

impl Default for Material {
    fn default() -> Material {
        Material::filled(Color::TRANSPARENT)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct BorderSide {
    pub color: Color,
    pub width: f32,
}

impl BorderSide {
    #[must_use]
    pub fn new(color: Color, width: f32) -> BorderSide {
        BorderSide { color, width }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct BorderRadius {
    top_left: f32,
    top_right: f32,
    bottom_right: f32,
    bottom_left: f32,
}

impl From<BorderRadius> for [f32; 4] {
    fn from(value: BorderRadius) -> Self {
        [
            value.top_left,
            value.top_right,
            value.bottom_right,
            value.bottom_left,
        ]
    }
}

#[derive(Clone, Copy, Default, PartialEq, Debug)]
pub struct BoxShadow {
    pub offset_left: f32,
    pub offset_top: f32,
    pub blur: f32,
    pub spread: f32,
    pub color: Color,
}

impl From<BoxShadow> for [f32; 4] {
    fn from(value: BoxShadow) -> Self {
        [
            value.offset_left,
            value.offset_top,
            value.blur,
            value.spread,
        ]
    }
}

impl BoxShadow {
    #[must_use]
    pub fn new(offset_left: f32, offset_top: f32, blur: f32, spread: f32, color: Color) -> Self {
        Self {
            offset_left,
            offset_top,
            blur,
            spread,
            color,
        }
    }
}

impl BorderRadius {
    #[must_use]
    pub fn new(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct Borders {
    pub top: Option<BorderSide>,
    pub bottom: Option<BorderSide>,
    pub left: Option<BorderSide>,
    pub right: Option<BorderSide>,
}

impl Borders {
    #[must_use]
    pub fn all(color: Color, width: f32) -> Borders {
        Borders {
            top: Some(BorderSide::new(color, width)),
            bottom: Some(BorderSide::new(color, width)),
            left: Some(BorderSide::new(color, width)),
            right: Some(BorderSide::new(color, width)),
        }
    }

    #[must_use]
    pub fn none() -> Borders {
        Borders {
            top: None,
            bottom: None,
            left: None,
            right: None,
        }
    }

    #[must_use]
    pub fn top(color: Color, width: f32) -> Borders {
        Borders {
            top: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn bottom(color: Color, width: f32) -> Borders {
        Borders {
            bottom: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn left(color: Color, width: f32) -> Borders {
        Borders {
            left: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn right(color: Color, width: f32) -> Borders {
        Borders {
            right: Some(BorderSide::new(color, width)),
            ..Borders::default()
        }
    }

    #[must_use]
    pub fn min(&self) -> Vector2 {
        let top = match self.top {
            Some(border) => border.width,
            None => 0.0,
        };
        let left = match self.left {
            Some(border) => border.width,
            None => 0.0,
        };
        Vector2::new(left, top)
    }

    #[must_use]
    pub fn max(&self) -> Vector2 {
        let top = match self.top {
            Some(border) => border.width,
            None => 0.0,
        };
        let right = match self.right {
            Some(border) => border.width,
            None => 0.0,
        };
        Vector2::new(right, top)
    }

    #[must_use]
    pub fn total_width(&self) -> f32 {
        let left = match self.left {
            Some(border) => border.width,
            None => 0.0,
        };
        let right = match self.right {
            Some(border) => border.width,
            None => 0.0,
        };
        left + right
    }

    #[must_use]
    pub fn total_height(&self) -> f32 {
        let top = match self.top {
            Some(border) => border.width,
            None => 0.0,
        };
        let bottom = match self.bottom {
            Some(border) => border.width,
            None => 0.0,
        };
        top + bottom
    }
}
