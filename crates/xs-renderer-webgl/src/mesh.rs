use crate::bindings::{
    GL_ARRAY_BUFFER, GL_NONE, GL_STATIC_DRAW, GL_TRIANGLES, GLsizei, GLsizeiptr, GLuint, GLvoid,
    glBindBuffer, glBufferData, glCreateBuffer, glDrawArrays,
};

/*
unsafe fn cast_slice(p: &[Vector3]) -> &[f32] {
    ::core::slice::from_raw_parts(
        (p as *const [Vector3]) as *const f32,
        ::core::mem::size_of::<Vector3>() / ::core::mem::size_of::<f32>() * p.len(),
    )
}
*/

pub struct Mesh {
    /// GL buffer
    pub buffer: GLuint,
    /// vertice number
    pub size: usize,
}

impl Mesh {
    pub fn new(vertices: &[f32]) -> Mesh {
        let size = vertices.len() / 3;
        unsafe {
            //let bytes = cast_slice(vertices);
            let buffer = glCreateBuffer();
            glBindBuffer(GL_ARRAY_BUFFER, buffer);
            glBufferData(
                GL_ARRAY_BUFFER,
                vertices.as_ptr() as _,
                (vertices.len() * 4) as _,
                GL_STATIC_DRAW,
            );
            glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
            Mesh { buffer, size }
        }
    }

    pub fn begin(&mut self) {
        unsafe {
            glBindBuffer(GL_ARRAY_BUFFER, self.buffer);
        }
    }

    pub fn end(&mut self) {
        unsafe {
            glDrawArrays(GL_TRIANGLES, 0, self.size as GLsizei);
            glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
        }
    }
}
