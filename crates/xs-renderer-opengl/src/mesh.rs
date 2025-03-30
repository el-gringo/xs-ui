use xs_math::Vector3;
use xs_gl_sys::{
    GLsizei,
    GLsizeiptr,
    GLuint,
    GLvoid,
    GL_ARRAY_BUFFER,
    GL_NONE,
    GL_STATIC_DRAW,
    GL_TRIANGLES,
    glBindBuffer,
    glBufferData,
    glDrawArrays,
    glGenBuffers,
};

use super::shader::ShaderGlsl;

unsafe fn cast_slice(p: &[Vector3]) -> &[f32] {
    ::core::slice::from_raw_parts(
        (p as *const [Vector3]) as *const f32,
        ::core::mem::size_of::<Vector3>() / 4 * p.len()
    )
}

pub struct Mesh {
    pub buffer: GLuint,
    pub count_vertices: usize
}

impl Mesh {
    pub fn new(vertices: &[Vector3]) -> Mesh {
        let mut buffer: GLuint = 0;
        unsafe {
            let bytes = cast_slice(vertices);
            glGenBuffers(1, &mut buffer);
            glBindBuffer(GL_ARRAY_BUFFER, buffer);
            glBufferData(
                GL_ARRAY_BUFFER,
                (bytes.len()  * std::mem::size_of::<f32>()) as GLsizeiptr,
                bytes.as_ptr() as *const GLvoid,
                GL_STATIC_DRAW
            );
            glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
            let count_vertices = vertices.len();
            Mesh { buffer, count_vertices }
        }
    }

    pub fn draw(&mut self, shader: &mut ShaderGlsl, uniform_values: Vec<(&str, *const u8)>) {
        unsafe {
            glBindBuffer(GL_ARRAY_BUFFER, self.buffer);
            shader.use_program();
            shader.apply_attributes();
            shader.apply_uniforms(uniform_values);
            glDrawArrays(
                GL_TRIANGLES,
                0,
                self.count_vertices as GLsizei,
            );
            glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
        }

    }
}
