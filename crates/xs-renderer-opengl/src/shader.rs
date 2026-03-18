use std::collections::HashMap;

use xs_gl_sys::{
    GL_COMPILE_STATUS, GL_FALSE, GL_FLOAT, GL_FRAGMENT_SHADER, GL_LINK_STATUS, GL_VERTEX_SHADER, GLenum, GLint, GLsizei, GLuint, glAttachShader, glCompileShader, glCreateProgram, glCreateShader, glEnableVertexAttribArray, glGenBuffers, glGetAttribLocation, glGetProgramInfoLog, glGetProgramParameter, glGetShaderInfoLog, glGetShaderParameter, glGetUniformLocation, glLinkProgram, glShaderSource, glUniform1f, glUniform1i, glUniform2f, glUniform4f, glUseProgram, glVertexAttribPointer
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

#[derive(Debug)]
pub struct ShaderGlslAttribute {
    pub buffer: GLuint,
    pub location: GLint,
    pub num_components: u8
}

pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    let program = glCreateProgram();
    glAttachShader(program, vertex_shader);
    glAttachShader(program, fragment_shader);
    glLinkProgram(program);

    let link_status = glGetProgramParameter(program, GL_LINK_STATUS);
    if link_status == 0 {
        let error_message = glGetProgramInfoLog(program);
        panic!("failed to link program: {:?}", error_message);
    }
    program
}

pub fn compile_shader(shader_type: GLenum, source: &str) -> GLuint {
    let shader = glCreateShader(shader_type);

    glShaderSource(shader, source);
    glCompileShader(shader);
    let compile_status = glGetShaderParameter(shader, GL_COMPILE_STATUS);
    if compile_status == 0 {
        let error_message = glGetShaderInfoLog(shader);
        panic!("failed to compile shader: {:?}\n{}", error_message, source);
    }
    shader
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
    ) -> ShaderGlsl {
        let vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_source);
        let fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_source);
        let program = link_program(vertex_shader, fragment_shader);
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
            let attribute = ShaderGlslAttribute { buffer, location, num_components: count as _ };
            attributes.insert(name.to_string(), attribute);
        }
        ShaderGlsl {
            program,
            uniforms,
            attributes,
        }
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

    pub fn apply_attributes(&self) {
        for (_name, attribute) in self.attributes.iter() {
            glEnableVertexAttribArray(attribute.location as _);
            glVertexAttribPointer(attribute.location as _, 3, GL_FLOAT, GL_FALSE as _, 0, 0);
        }
    }

    pub fn use_program(&self) {
        glUseProgram(self.program);
    }
}
