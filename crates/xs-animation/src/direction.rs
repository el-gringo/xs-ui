#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum AnimationDirection {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}
