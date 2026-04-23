use xs_gl_sys::{
    GL_ARRAY_BUFFER, GL_STATIC_DRAW, glBindBuffer, glBufferData, glEnableVertexAttribArray,
    glGenBuffers,
};

use crate::attribute::Attribute;

pub struct Buffer {
    buffer: u32,
    pub attributes: Vec<Attribute>,
}

impl Buffer {
    pub fn id(&self) -> u32 {
        self.buffer
    }
}

impl Default for Buffer {
    fn default() -> Self {
        let buffer = glGenBuffers(1)[0];
        Self {
            buffer,
            attributes: Default::default(),
        }
    }
}

impl Buffer {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn bind(&self) {
        glBindBuffer(GL_ARRAY_BUFFER, self.buffer);
        for attr in self.attributes.iter() {
            glEnableVertexAttribArray(attr.location as _);
        }
    }

    pub fn with_attribute(&mut self, mut callback: impl FnMut(&mut Attribute)) {
        for attr in self.attributes.iter_mut() {
            callback(attr);
        }
    }

    pub fn push_data(&self, data: &[f32]) {
        glBindBuffer(GL_ARRAY_BUFFER, self.buffer);
        glBufferData(GL_ARRAY_BUFFER, data, GL_STATIC_DRAW);
    }

    pub fn add_attribute(
        mut self,
        name: &'static str,
        size: i32,
        r#type: u32,
        normalized: bool,
        stride: i32,
        offset: i64,
    ) -> Self {
        let attribute = Attribute::new(name, -1, size, r#type, normalized, stride, offset);
        self.attributes.push(attribute);
        self
    }
}
