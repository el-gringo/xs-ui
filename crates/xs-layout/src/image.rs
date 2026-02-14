#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct Image(pub i32);

impl Default for Image {
    fn default() -> Self {
        Self(-1)
    }
}

impl Image {
    pub fn as_ptr(&self) -> i32 {
        self.0
    }
}
