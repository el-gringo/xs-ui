#[derive(Clone)]
pub struct Attribute {
    pub location: i32,
    pub name: &'static str,
    pub size: i32,
    pub r#type: u32,
    pub normalized: bool,
    pub stride: i32,
    pub offset: i64,
}

impl Attribute {
    pub fn new(
        name: &'static str,
        location: i32,
        size: i32,
        r#type: u32,
        normalized: bool,
        stride: i32,
        offset: i64,
    ) -> Self {
        Self {
            name,
            location,
            size,
            r#type,
            normalized,
            stride,
            offset,
        }
    }
}
