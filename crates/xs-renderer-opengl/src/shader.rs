use std::collections::HashMap;

use xs_gl::{
    GL_FRAGMENT_SHADER, GL_VERTEX_SHADER, GLint, GLsizei, GLuint, compile_shader, glGenBuffers,
    glGetAttribLocation, glGetUniformLocation, glUniform1f, glUniform1i, glUniform2f, glUniform4f,
    glUseProgram, link_program,
};

#[allow(dead_code)]
#[derive(Clone)]
pub enum ShaderGlslFormat {
    /// One 32-bit wide float (equivalent to `f32`)
    Float1,
    /// Two 32-bit wide floats (equivalent to `[f32; 2]`)
    Float2,
    /// Three 32-bit wide floats (equivalent to `[f32; 3]`)
    Float3,
    /// Four 32-bit wide floats (equivalent to `[f32; 4]`)
    Float4,
    /// One unsigned 32-bit integers (equivalent to `[u32; 1]`)
    Int1,
    /// Two unsigned 32-bit integers (equivalent to `[u32; 2]`)
    Int2,
    /// Three unsigned 32-bit integers (equivalent to `[u32; 3]`)
    Int3,
    /// Four unsigned 32-bit integers (equivalent to `[u32; 4]`)
    Int4,
    /// Four by four matrix of 32-bit floats
    Mat4,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ShaderGlslUniform {
    pub location: GLint,
    format: ShaderGlslFormat,
    count: GLsizei,
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct ShaderGlslAttribute {
    pub buffer: GLuint,
    pub location: GLint,
    pub num_components: u8,
}

pub struct ShaderGlsl {
    pub program: GLuint,
    pub uniforms: HashMap<String, ShaderGlslUniform>,
    pub attributes: HashMap<String, ShaderGlslAttribute>,
}

impl ShaderGlsl {
    pub fn new(
        vertex_source: &str,
        fragment_source: &str,
        uniform_descriptions: Vec<(String, (ShaderGlslFormat, i32))>,
        attribute_descriptions: Vec<(String, (ShaderGlslFormat, i32))>,
    ) -> Result<ShaderGlsl, String> {
        let vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_source)?;
        let fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_source)?;
        let program = link_program(vertex_shader, fragment_shader)?;
        glUseProgram(program);
        let mut uniforms = HashMap::new();
        for (name, (format, count)) in uniform_descriptions {
            let location = glGetUniformLocation(program, &name);
            let uniform = ShaderGlslUniform {
                location,
                format,
                count,
            };
            uniforms.insert(name, uniform);
        }
        let mut attributes = HashMap::new();
        for (name, (_format, count)) in attribute_descriptions {
            let buffer = glGenBuffers(1)[0];
            let location = glGetAttribLocation(program, &name);
            let attribute = ShaderGlslAttribute {
                buffer,
                location,
                num_components: count as _,
            };
            attributes.insert(name.to_string(), attribute);
        }
        Ok(ShaderGlsl {
            program,
            uniforms,
            attributes,
        })
    }

    /*
    pub fn paint(&mut self, uniform_values: Vec<(String, *const u8)>) {
        self.use_program();
        self.apply_attributes();
        self.apply_uniforms(uniform_values);
    }

    pub fn set_uniform_value(&mut self, name: String, value: *const u8) {
        if let Some(uniform) = self.uniforms.get(&name) {
            unsafe {
                match uniform.format {
                    Float1  => glUniform1fv(uniform.location, uniform.count, value as *const f32),
                    Float2  => glUniform2fv(uniform.location, uniform.count, value as *const f32),
                    Float3  => glUniform3fv(uniform.location, uniform.count, value as *const f32),
                    Float4  => glUniform4fv(uniform.location, uniform.count, value as *const f32),
                    Int1    => glUniform1iv(uniform.location, uniform.count, value as *const i32),
                    Int2    => glUniform2iv(uniform.location, uniform.count, value as *const i32),
                    Int3    => glUniform3iv(uniform.location, uniform.count, value as *const i32),
                    Int4    => glUniform4iv(uniform.location, uniform.count, value as *const i32),
                    Mat4    => {}
                }
            }
        }
    }

    pub fn apply_uniforms(&mut self, uniform_values: Vec<(String, *const u8)>) {
        for (name, value) in uniform_values {
            self.set_uniform_value(name, value);
        }
    }
    */

    #[allow(dead_code)]
    pub fn set_uniform_1f(&self, name: &str, v0: f32) {
        if let Some(uniform) = self.uniforms.get(name) {
            glUniform1f(uniform.location, v0);
        }
    }

    pub fn set_uniform_2f(&self, name: &str, v0: f32, v1: f32) {
        if let Some(uniform) = self.uniforms.get(name) {
            glUniform2f(uniform.location, v0, v1);
        }
    }

    /*
    pub fn set_uniform_3f(&mut self, name: &str, v0: f32, v1: f32, v2: f32) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe { glUniform3f(uniform.location, v0, v1, v2); }
        }
    }
    */

    pub fn set_uniform_4f(&self, name: &str, v0: f32, v1: f32, v2: f32, v3: f32) {
        if let Some(uniform) = self.uniforms.get(name) {
            glUniform4f(uniform.location, v0, v1, v2, v3);
        }
    }

    pub fn set_uniform_1i(&self, name: &str, v0: i32) {
        if let Some(uniform) = self.uniforms.get(name) {
            glUniform1i(uniform.location, v0);
        }
    }

    /*
    pub fn set_uniform_2i(&mut self, name: &str, v0: i32, v1: i32) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe { glUniform2i(uniform.location, v0, v1); }
        }
    }

    pub fn set_uniform_3i(&mut self, name: &str, v0: i32, v1: i32, v2: i32) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe { glUniform3i(uniform.location, v0, v1, v2); }
        }
    }

    pub fn set_uniform_4i(&mut self, name: &str, v0: i32, v1: i32, v2: i32, v3: i32) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe { glUniform4i(uniform.location, v0, v1, v2, v3); }
        }
    }
    */

    pub fn use_program(&self) {
        glUseProgram(self.program);
    }
}
