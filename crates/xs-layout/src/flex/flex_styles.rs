#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlexDirection {
    Column,
    ColumnReverse,
    Row,
    RowReverse,
}

impl Default for FlexDirection {
    fn default() -> Self {
        Self::Row
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum FlexWrap {
    NoWrap,
    WrapReverse,
    Wrap,
}

impl Default for FlexWrap {
    fn default() -> Self {
        Self::NoWrap
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AlignContent {
    Center,
    FlexEnd,
    FlexStart,
    SpaceAround,
    SpaceBetween,
    SpaceEvenly,
    Stretch,
}

impl Default for AlignContent {
    fn default() -> Self {
        Self::FlexStart
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum AlignItems {
    Baseline,
    Center,
    FlexEnd,
    FlexStart,
    Stretch,
}

impl Default for AlignItems {
    fn default() -> Self {
        Self::FlexStart
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum JustifyContent {
    Center,
    FlexEnd,
    FlexStart,
    SpaceAround,
    SpaceBetween,
    SpaceEvenly,
}

impl Default for JustifyContent {
    fn default() -> Self {
        Self::FlexStart
    }
}
