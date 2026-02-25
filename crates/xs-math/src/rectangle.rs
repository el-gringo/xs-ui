use crate::Vector2;

/// A 2-dimensional rectangle.
#[derive(PartialEq, Copy, Clone, Default, Debug)]
pub struct Rectangle {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rectangle {
    /// Construct a new vector that starts at the point described by `min` and
    /// ends at the point described by `max`.
    #[must_use]
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    #[must_use]
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0, width: 0.0, height: 0.0 }
    }


    #[must_use]
    pub fn from_pos<I: Into<Vector2>>(pos: I, size: I) -> Self {
        let pos: Vector2 = pos.into();
        let size: Vector2 = size.into();
        Self { x: pos.x, y: pos.y, width: pos.x + size.x, height: pos.y + size.y }
    }

    #[must_use]
    pub fn from_size<I: Into<Vector2>>(max: I) -> Self {
        let max: Vector2 = max.into();
        Self { x: 0.0, y: 0.0, width: max.x, height: max.y }
    }

    #[must_use]
    pub fn pos(self) -> Vector2 {
        Vector2::new(self.x, self.y)
    }

    /// Get the width and height of the rectangle.
    #[must_use]
    pub fn size(self) -> Vector2 {
        Vector2::new(self.width, self.height)
    }

    /// Move the rectangle by the provided amount.
    #[must_use]
    pub fn translate(self, amount: Vector2) -> Self {
        Self { x: self.x + amount.x, y: self.y + amount.y, width: self.width, height: self.height }
    }

    /// Check if the rectangle intersects a point.
    #[must_use]
    pub fn intersects(self, point: Vector2) -> bool {
        let intersects_x = self.x < point.x && point.x < self.x + self.width;
        let intersects_y = self.y < point.y && point.y < self.y + self.height;
        intersects_x && intersects_y
    }
}
