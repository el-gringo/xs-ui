#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum FlexDirection {
    Column,
    ColumnReverse,
    #[default]
    Row,
    RowReverse,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum FlexWrap {
    #[default]
    NoWrap,
    WrapReverse,
    Wrap,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum AlignContent {
    Center,
    FlexEnd,
    #[default]
    FlexStart,
    SpaceAround,
    SpaceBetween,
    SpaceEvenly,
    Stretch,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum AlignItems {
    Baseline,
    Center,
    FlexEnd,
    #[default]
    FlexStart,
    Stretch,
}

#[derive(Copy, Clone, PartialEq, Debug, Default)]
pub enum JustifyContent {
    Center,
    FlexEnd,
    #[default]
    FlexStart,
    SpaceAround,
    SpaceBetween,
    SpaceEvenly,
}
