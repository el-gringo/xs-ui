use std::{collections::HashMap, ffi::CString};

use xs_gl_sys::{
    glAttachShader, glCompileShader, glCreateProgram, glCreateShader, glEnableVertexAttribArray,
    glGetAttribLocation, glGetProgramInfoLog, glGetProgramiv, glGetShaderInfoLog, glGetShaderiv,
    glGetUniformLocation, glLinkProgram, glShaderSource, glUniform1f, glUniform1i, glUniform2f,
    glUniform4f, glUseProgram, glVertexAttribPointer, GLenum, GLint, GLsizei, GLuint,
    GL_COMPILE_STATUS, GL_FALSE, GL_FLOAT, GL_FRAGMENT_SHADER, GL_INFO_LOG_LENGTH, GL_LINK_STATUS,
    GL_VERTEX_SHADER,
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

pub struct ShaderGlslAttribute {
    pub location: GLint,
}

pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    unsafe {
        let program = glCreateProgram();
        glAttachShader(program, vertex_shader);
        glAttachShader(program, fragment_shader);
        glLinkProgram(program);

        let mut link_status = 0;
        glGetProgramiv(program, GL_LINK_STATUS, &mut link_status as *mut _);
        if link_status == 0 {
            let mut max_length: i32 = 0;
            glGetProgramiv(program, GL_INFO_LOG_LENGTH, &mut max_length as *mut _);
            let mut error_message = vec![0u8; max_length as usize + 1];
            glGetProgramInfoLog(
                program,
                max_length,
                &mut max_length as *mut _,
                error_message.as_mut_ptr() as *mut _,
            );
            assert!(max_length >= 1);
            let error_message =
                std::string::String::from_utf8_lossy(&error_message[0..max_length as usize - 1]);
            panic!("failed to link program: {:?}", error_message);
        }
        program
    }
}

pub fn compile_shader(shader_type: GLenum, source: &str) -> GLuint {
    unsafe {
        let shader = glCreateShader(shader_type);
        if let Ok(csource) = CString::new(source) {
            glShaderSource(shader, 1, [csource].as_ptr() as *const _, std::ptr::null());
            glCompileShader(shader);
            let mut compile_status = 0;
            glGetShaderiv(shader, GL_COMPILE_STATUS, &mut compile_status as *mut _);
            if compile_status == 0 {
                let mut max_length: i32 = 0;
                glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut max_length as *mut _);
                let mut error_message = vec![0u8; max_length as usize + 1];
                glGetShaderInfoLog(
                    shader,
                    max_length,
                    &mut max_length as *mut _,
                    error_message.as_mut_ptr() as *mut _,
                );
                panic!("failed to compile shader: {:?}", error_message);
            }
            return shader;
        }
        panic!("compile_shader failed");
    }
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
        let uniforms = unsafe {
            glUseProgram(program);
            let mut uniforms = HashMap::new();
            for (name, (format, count)) in uniform_descriptions {
                let cname = CString::new(name.clone()).unwrap_or_else(|e| panic!("{}", e));
                let location = glGetUniformLocation(program, cname.as_ptr() as *const _);
                let uniform = ShaderGlslUniform {
                    location,
                    format,
                    count,
                };
                uniforms.insert(name, uniform);
            }
            uniforms
        };
        let attributes = unsafe {
            let mut attributes = HashMap::new();
            for (name, (_format, _count)) in attribute_descriptions {
                let cname = CString::new(name.clone()).unwrap_or_else(|e| panic!("{}", e));
                let location = glGetAttribLocation(program, cname.as_ptr() as *const _);
                let attribute = ShaderGlslAttribute { location };
                attributes.insert(name.to_string(), attribute);
            }
            attributes
        };
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
            unsafe {
                glUniform1f(uniform.location, v0);
            }
        }
    }

    pub fn set_uniform_2f(&self, name: &str, v0: f32, v1: f32) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe {
                glUniform2f(uniform.location, v0, v1);
            }
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
            unsafe {
                glUniform4f(uniform.location, v0, v1, v2, v3);
            }
        }
    }

    pub fn set_uniform_1i(&self, name: &str, v0: i32) {
        if let Some(uniform) = self.uniforms.get(name) {
            unsafe {
                glUniform1i(uniform.location, v0);
            }
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

    pub fn use_program(&self) {
        unsafe {
            glUseProgram(self.program);
        }
    }
}
