//use bytemuck::cast_slice;
use crate::bindings::*;
use xs_layout::{Color, Image};
use xs_math::Vector3;

fn cast_slice(p: &[Vector3]) -> &[f32] {
    unsafe {
        ::core::slice::from_raw_parts(
            (p as *const [Vector3]) as *const f32,
            ::core::mem::size_of::<Vector3>() / 4 * p.len(),
        )
    }
}

pub fn clear(color: Color) {
    unsafe {
        glClearColor(
            color.r as GLfloat,
            color.g as GLfloat,
            color.b as GLfloat,
            color.a as GLfloat,
        );
        glClear(GL_COLOR_BUFFER_BIT);
    }
}

pub fn set_viewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    unsafe {
        glViewport(x, y, width, height);
    }
}

// #[allow(dead_code)]
// pub fn draw_line(
//     program: GLuint,
//     vertex_attribute_name: &str,
//     buffer: &Buffer,
// ) {
//     unsafe {
//         glUseProgram(program);
//         glBindBuffer(GL_ARRAY_BUFFER, buffer.buffer);
//         let location = glGetAttribLocation(
//             program,
//             vertex_attribute_name.as_ptr() as *const GLchar,
//             vertex_attribute_name.len() as u32
//         );
//         glVertexAttribPointer(
//             location as u32,
//             buffer.element_size as i32,
//             GL_FLOAT,
//             GL_FALSE as u8,
//             0,
//             0 as *const ::core::ffi::c_void,
//         );
//         glEnableVertexAttribArray(location as GLuint);
//         glDrawArrays(
//             GL_LINES,
//             0,
//             (buffer.len / buffer.element_size) as i32,
//         );
//         glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
//     }
// }

pub fn draw_mesh(program: GLuint, vertex_attribute_name: &str, vertices: Vec<Vector3>) {
    unsafe {
        /*
        glEnable(GL_BLEND);
        glBlendFunc(
            GL_SRC_ALPHA,
            GL_ONE_MINUS_SRC_ALPHA,
        );
        */
        let vertice_buffer = new_array_buffer(&vertices);
        glBindBuffer(GL_ARRAY_BUFFER, vertice_buffer.buffer);

        // let texture_coordinates = [
        //     // Front
        //     0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        //     // Back
        //     0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        //     // Top
        //     0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        //     // Bottom
        //     0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        //     // Right
        //     0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        //     // Left
        //     0.0, 0.0, 1.0, 0.0, 1.0, 1.0, 0.0, 1.0,
        //   ];

        // let texture_coordinates_buffer = new_array_buffer(texture_coordinates.to_vec());
        // glBindBuffer(GL_ARRAY_BUFFER, texture_coordinates_buffer.buffer);

        // Upload to GPU
        glUseProgram(program);
        let location = glGetAttribLocation(
            program,
            vertex_attribute_name.as_ptr() as *const GLchar,
            vertex_attribute_name.len() as u32,
        );
        glVertexAttribPointer(
            location as u32,
            vertice_buffer.element_size as i32,
            GL_FLOAT,
            GL_FALSE as u8,
            0,
            0 as *const ::core::ffi::c_void,
        );
        glEnableVertexAttribArray(location as u32);
        glDrawArrays(
            GL_TRIANGLES,
            0,
            (vertice_buffer.len / vertice_buffer.element_size) as i32,
        );
        glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
    }
}

pub fn new_array_buffer(values: &[Vector3]) -> Buffer {
    let bytes: &[f32] = cast_slice(values);
    let buffer = unsafe {
        let buffer = glCreateBuffer();
        glBindBuffer(GL_ARRAY_BUFFER, buffer);
        glBufferData(
            GL_ARRAY_BUFFER,
            bytes.as_ptr() as *const _,
            bytes.len() as GLsizeiptr,
            GL_STATIC_DRAW,
        );
        glBindBuffer(GL_ARRAY_BUFFER, GL_NONE);
        Buffer {
            buffer,
            len: bytes.len() as u32,
            element_size: 3,
        }
    };
    buffer
}

pub fn set_uniform_i32(program: GLuint, field: &str, value: i32) {
    unsafe {
        glUseProgram(program);
        let location =
            glGetUniformLocation(program, field.as_ptr() as *const _, field.len() as u32);
        glUniform1i(location, value);
    }
}

pub fn set_uniform_f32(program: GLuint, field: &str, value: f32) {
    unsafe {
        glUseProgram(program);
        let location =
            glGetUniformLocation(program, field.as_ptr() as *const _, field.len() as u32);
        glUniform1f(location, value);
    }
}

pub fn set_uniform_vec2(program: GLuint, field: &str, value: [f32; 2]) {
    unsafe {
        glUseProgram(program);
        let location =
            glGetUniformLocation(program, field.as_ptr() as *const _, field.len() as u32);
        glUniform2fv(
            location,
            value.len() as i32,
            value.as_ptr() as *const _,
            field.len() as u32,
        );
    }
}

pub fn set_uniform_vec4(program: GLuint, field: &str, value: [f32; 4]) {
    unsafe {
        glUseProgram(program);
        let location =
            glGetUniformLocation(program, field.as_ptr() as *const _, field.len() as u32);
        glUniform4fv(
            location,
            value.len() as i32,
            value.as_ptr() as *const _,
            field.len() as u32,
        );
    }
}

pub fn set_uniform1i(program: GLuint, field: &str, value: i32) {
    unsafe {
        glUseProgram(program);
        let location =
            glGetUniformLocation(program, field.as_ptr() as *const _, field.len() as u32);
        glUniform1i(location, value);
    }
}

/// This will try to create a shader program that uses the provided vertex and
/// fragment shader sourcecodes.
pub fn create_shader_program(vertex_shader_src: &str, fragment_shader_src: &str) -> GLuint {
    let vertex_shader = compile_shader(GL_VERTEX_SHADER, vertex_shader_src);
    let fragment_shader = compile_shader(GL_FRAGMENT_SHADER, fragment_shader_src);
    link_program(vertex_shader, fragment_shader)
}

/// Try to link the shader program. If linking fails, this will attempt to get
/// the error message from the program info log. If that also fails, it
/// will return a generic error.
pub fn link_program(vertex_shader: GLuint, fragment_shader: GLuint) -> GLuint {
    unsafe {
        let program = glCreateProgram();
        glAttachShader(program, vertex_shader);
        glAttachShader(program, fragment_shader);
        glLinkProgram(program);
        /*
        let maybe_link_status = glGetProgramParameter(program, GL_LINK_STATUS);
        if maybe_link_status != 0 {
            let err = glGetProgramInfoLog(program, bufSize, length, infoLog);
            return Err(err);
        }
        */
        program
    }
}

/// Attempt to compile the shader. If compilation fails, this will attempt to
/// get the error message from the shader info log. If that also fails, it will
/// return a generic error.
pub fn compile_shader(shader_type: u32, source: &str) -> GLuint {
    unsafe {
        let shader = glCreateShader(shader_type);
        glShaderSource(
            shader,
            source.as_ptr() as *const GLchar,
            source.len() as i32,
        );
        glCompileShader(shader);
        /*
        let maybe_link_status = glGetProgramParameter(shader, GL_COMPILE_STATUS);
        if maybe_link_status != 0 {
            let err = glGetProgramInfoLog(program, bufSize, length, infoLog);
            return Err(err);
        }
        */
        shader
    }
}

fn is_power_of_2(value: u32) -> bool {
    (value & (value - 1)) == 0
}

pub fn load_texture(
    rgba: &[u8],
    image_handle: u32,
    width: u32,
    height: u32,
    alpha: bool,
) -> GLuint {
    unsafe {
        let texture = glCreateTexture();
        glActiveTexture(texture);
        glBindTexture(GL_TEXTURE_2D, texture);
        glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        glTexImage2D(
            GL_TEXTURE_2D,
            0,
            GL_RGBA,
            width as _,
            height as _,
            0,
            GL_RGBA,
            GL_UNSIGNED_BYTE,
            if rgba.len() > 0 {
                rgba.as_ptr() as _
            } else {
                image_handle as _
            },
        );
        if is_power_of_2(width) && is_power_of_2(height) {
            glGenerateMipmap(GL_TEXTURE_2D);
        } else {
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_LINEAR);
        }
        texture
    }
}

/// Represents an arbitrary amount of data held in the GPU.
#[derive(Clone)]
pub struct Buffer {
    /// The [WebGlBuffer] that holds the data.
    pub buffer: GLuint,
    /// The number of elements in the buffer.
    pub len: u32,
    /// The number of elements per type. For example, Vector2 is size 2.
    pub element_size: u32,
}
