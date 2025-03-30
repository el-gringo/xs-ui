use std::{collections::HashMap, ffi::CString};

use xs_gl_sys::{
    create_shader_program, glEnableVertexAttribArray, glGetAttribLocation, glGetUniformLocation, glUniform1fv, glUniform1iv, glUniform2fv, glUniform2iv, glUniform3fv, glUniform3iv, glUniform4fv, glUniform4iv, glUseProgram, glVertexAttribPointer, GLint, GLsizei, GLuint, GL_FALSE, GL_FLOAT
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

#[derive(Clone)]
pub struct ShaderGlslUniform {
    pub location: GLint,
    format: ShaderGlslFormat,
    count: GLsizei,
}

pub struct ShaderGlslAttribute {
    pub location: GLint,
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
        uniform_descriptions: Vec<(&str, (ShaderGlslFormat, i32))>,
        attribute_descriptions: Vec<(&str, (ShaderGlslFormat, i32))>,
    ) -> ShaderGlsl {
        let program = create_shader_program(vertex_source, fragment_source);
        let uniforms = unsafe {
            glUseProgram(program);
            let mut uniforms = HashMap::new();
            for (name, (format, count)) in uniform_descriptions {
                let cname = CString::new(name).unwrap_or_else(|e| panic!("{}", e));
                let location = glGetUniformLocation(program, cname.as_ptr() as *const _);
                let uniform = ShaderGlslUniform {
                    location,
                    format,
                    count,
                };
                uniforms.insert(name.to_string(), uniform);
            }
            uniforms
        };
        let attributes = unsafe {
            glUseProgram(program);
            let mut attributes = HashMap::new();
            for (name, (_format, _count)) in attribute_descriptions {
                let cname = CString::new(name).unwrap_or_else(|e| panic!("{}", e));
                let location = glGetAttribLocation(program, cname.as_ptr() as *const _);
                let uniform = ShaderGlslAttribute { location };
                attributes.insert(name.to_string(), uniform);
            }
            attributes
        };
        ShaderGlsl {
            program,
            uniforms,
            attributes,
        }
    }

    pub fn set_uniform_value(&mut self, name: &str, value: *const u8) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe {
                match uniform.format {
                    ShaderGlslFormat::Float1 => {
                        glUniform1fv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Float2 => {
                        glUniform2fv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Float3 => {
                        glUniform3fv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Float4 => {
                        glUniform4fv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Int1 => {
                        glUniform1iv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Int2 => {
                        glUniform2iv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Int3 => {
                        glUniform3iv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Int4 => {
                        glUniform4iv(uniform.location, uniform.count, value as _);
                    }
                    ShaderGlslFormat::Mat4 => {}
                }
            }
        }
    }

    pub fn apply_uniforms(&mut self, uniform_values: Vec<(&str, *const u8)>) {
        for (name, value) in uniform_values {
            self.set_uniform_value(name, value);
        }
    }

    pub fn apply_attributes(&mut self) {
        for (_name, attribute) in self.attributes.iter() {
            unsafe {
                glEnableVertexAttribArray(attribute.location as _);
                glVertexAttribPointer(
                    attribute.location as _,
                    3,
                    GL_FLOAT,
                    GL_FALSE as _,
                    0,
                    std::ptr::null(),
                );
            }
        }
    }

    pub fn use_program(&mut self) {
        unsafe {
            glUseProgram(self.program);
        }
    }
}
