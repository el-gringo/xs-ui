use std::collections::{BTreeMap, HashMap};

use xs_gl_sys::{
    GL_ARRAY_BUFFER, GL_COMPILE_STATUS, GL_FRAGMENT_SHADER, GL_LINK_STATUS, GL_STATIC_DRAW,
    GL_VERTEX_SHADER, GLenum, GLuint, glAttachShader, glBindBuffer, glBufferData, glCompileShader,
    glCreateProgram, glCreateShader, glEnableVertexAttribArray, glGenBuffers, glGetAttribLocation,
    glGetProgramInfoLog, glGetProgramParameter, glGetShaderInfoLog, glGetShaderParameter,
    glGetUniformLocation, glLinkProgram, glShaderSource, glUniform1f, glUniform1i, glUniform2f,
    glUniform3f, glUniform4f, glUniform4fv, glUniformMatrix3fv, glUniformMatrix4fv, glUseProgram,
    glVertexAttribPointer,
};

pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> Result<GLuint, String> {
    let program = glCreateProgram();
    glAttachShader(program, vertex_shader);
    glAttachShader(program, fragment_shader);
    glLinkProgram(program);

    let link_status = glGetProgramParameter(program, GL_LINK_STATUS);
    if link_status == 0 {
        let error_message = glGetProgramInfoLog(program);
        eprintln!("Failed to link program: {}", error_message);
        return Err("failed to link program: ".to_owned() + &error_message);
    }
    Ok(program)
}

pub fn compile_shader(shader_type: GLenum, source: &str) -> Result<GLuint, String> {
    let shader = glCreateShader(shader_type);

    glShaderSource(shader, source);
    glCompileShader(shader);
    let compile_status = glGetShaderParameter(shader, GL_COMPILE_STATUS);
    if compile_status == 0 {
        let error_message = glGetShaderInfoLog(shader);
        let shader_type = match shader_type {
            GL_VERTEX_SHADER => "vertex",
            GL_FRAGMENT_SHADER => "fragment",
            _ => "unknown",
        };
        eprintln!(
            "Failed to compile {} shader: {}",
            shader_type, error_message
        );
        eprintln!("Shader source: {}", source);
        return Err("failed to compile ".to_owned() + shader_type + " shader: " + &error_message);
    }
    Ok(shader)
}

type AttributeBuffer = u32;
type AttributeLocation = u32;
type AttributeName = &'static str;
type AttributeSize = i32;
type AttributeType = u32;
type AttributeNormalized = bool;
type AttributeStride = i32;
type AttributeOffset = i64;

#[derive(Clone)]
pub struct AttributeInfo {
    buffer: AttributeBuffer,
    location: AttributeLocation,
    size: AttributeSize,
    r#type: AttributeType,
    normalized: AttributeNormalized,
    stride: AttributeStride,
    offset: AttributeOffset,
}

impl AttributeInfo {
    pub fn new(
        buffer: AttributeBuffer,
        location: AttributeLocation,
        size: AttributeSize,
        r#type: AttributeType,
        normalized: AttributeNormalized,
        stride: AttributeStride,
        offset: AttributeOffset,
    ) -> Self {
        Self {
            buffer,
            location,
            size,
            r#type,
            normalized,
            stride,
            offset,
        }
    }
}

type AttributeInline = (
    AttributeName,
    AttributeSize,
    AttributeType,
    AttributeNormalized,
    AttributeStride,
    AttributeOffset,
);

pub struct ShaderGlSl {
    program: u32,
    uniforms: HashMap<&'static str, i32>,
    attributes: Vec<BTreeMap<&'static str, AttributeInfo>>,
}

impl ShaderGlSl {
    pub fn try_new(
        vertex_source: &str,
        fragment_source: &str,
        attributes_groups: &[&[AttributeInline]],
    ) -> Result<Self, String> {
        let vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_source)?;
        let fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_source)?;
        let program = link_program(vertex_shader, fragment_shader)?;
        let uniforms = HashMap::new();
        // for name in uniform_names {
        //     let location = glGetUniformLocation(program, name);
        //     eprintln!("Uniform {} location: {}", name, location);
        //     if location == -1 {
        //         eprintln!("Warning: Uniform '{}' not found in shader!", name);
        //     }
        //     uniforms.insert(*name, location);
        // }
        let mut attributes = Vec::new();
        for attribute_group in attributes_groups {
            let buffer = *glGenBuffers(1).first().unwrap();
            let mut group = BTreeMap::new();
            for (name, size, r#type, normalized, stride, offset) in *attribute_group {
                let location = glGetAttribLocation(program, name);
                group.insert(
                    *name,
                    AttributeInfo::new(
                        buffer,
                        location as _,
                        *size,
                        *r#type,
                        *normalized,
                        *stride,
                        *offset,
                    ),
                );
            }
            attributes.push(group);
        }
        Ok(Self {
            program,
            uniforms,
            attributes,
        })
    }

    fn get_uniform_location(&mut self, name: &'static str) -> i32 {
        *self.uniforms.entry(name).or_insert_with(|| {
            let location = glGetUniformLocation(self.program, name);
            if location == -1 {
                eprintln!("ERROR: Uniform '{}' not found in shader!", name);
            } else {
                eprintln!("Uniform {} location: {}", name, location);
            }
            location
        })
    }

    pub fn attribute_buffer_data(&self, attribute_name: &'static str, data: &[f32]) {
        let group = self
            .attributes
            .iter()
            .find(|group| group.contains_key(attribute_name))
            .unwrap_or_else(|| panic!("GLSL attribute not found {}", attribute_name));
        let first_attribute = group.values().next().unwrap();
        glBindBuffer(GL_ARRAY_BUFFER, first_attribute.buffer);
        glBufferData(GL_ARRAY_BUFFER, data, GL_STATIC_DRAW);
        for attr in group.values() {
            glEnableVertexAttribArray(attr.location as _);
            glVertexAttribPointer(
                attr.location,
                attr.size,
                attr.r#type,
                attr.normalized as _,
                attr.stride,
                attr.offset,
            );
        }
    }

    pub fn use_program(&self) {
        glUseProgram(self.program);
    }

    pub fn uniform_1f(&mut self, name: &'static str, v0: f32) {
        let location = self.get_uniform_location(name);
        glUniform1f(location, v0);
    }

    pub fn uniform_1i(&mut self, name: &'static str, v0: i32) {
        let location = self.get_uniform_location(name);
        glUniform1i(location, v0);
    }

    pub fn uniform_2f(&mut self, name: &'static str, v0: f32, v1: f32) {
        let location = self.get_uniform_location(name);
        glUniform2f(location, v0, v1);
    }

    pub fn uniform_3f(&mut self, name: &'static str, v0: f32, v1: f32, v2: f32) {
        let location = self.get_uniform_location(name);
        glUniform3f(location, v0, v1, v2);
    }

    pub fn uniform_4f(&mut self, name: &'static str, v0: f32, v1: f32, v2: f32, v3: f32) {
        let location = self.get_uniform_location(name);
        glUniform4f(location, v0, v1, v2, v3);
    }

    pub fn uniform_4fv(&mut self, name: &'static str, value: impl Into<[f32; 4]>) {
        let location = self.get_uniform_location(name);
        glUniform4fv(location, &value.into());
    }

    pub fn uniform_matrix_3fv<'a>(
        &mut self,
        name: &'static str,
        transpose: bool,
        value: impl Into<&'a [f32; 9]>,
    ) {
        let location = self.get_uniform_location(name);
        glUniformMatrix3fv(location, transpose, value.into());
    }

    pub fn uniform_matrix_4fv(
        &mut self,
        name: &'static str,
        transpose: bool,
        value: impl Into<[f32; 16]>,
    ) {
        let location = self.get_uniform_location(name);
        glUniformMatrix4fv(location, transpose, &value.into());
    }
}
