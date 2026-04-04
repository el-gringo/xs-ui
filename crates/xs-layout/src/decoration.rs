use crate::{Color, Image, TextVertices, colors::TRANSPARENT};
use xs_math::Vector2;

#[derive(Clone, PartialEq, Debug)]
pub enum Material {
    Quad(Borders, Color, Option<Image>),
    Text(Color, (String, u16, TextVertices)),
}

impl Material {
    #[must_use]
    pub fn filled(color: Color) -> Material {
        Material::Quad(Borders::none(), color, None)
    }
}

impl Default for Material {
    fn default() -> Material {
        Material::filled(TRANSPARENT)
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
            top: Some(BorderSide::new(color.clone(), width)),
            bottom: Some(BorderSide::new(color.clone(), width)),
            left: Some(BorderSide::new(color.clone(), width)),
            right: Some(BorderSide::new(color.clone(), width)),
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
