#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use std::ffi::CString;

pub use crate::bindings::load_gl_funcs;
use crate::bindings::{self, common_sys::*};

pub fn glBufferData(
    target: GLenum,
    size: GLsizeiptr,
    data: *const ::core::ffi::c_void,
    usage: GLenum,
) {
    unsafe {
        bindings::glBufferData(target, size, data, usage);
    }
}

pub fn glVertexAttribPointer(
    index: GLuint,
    size: GLint,
    type_: GLenum,
    normalized: GLboolean,
    stride: GLsizei,
    _pointer: GLintptr,
) {
    unsafe {
        bindings::glVertexAttribPointer(index, size, type_, normalized, stride, std::ptr::null());
    }
}

pub fn glGenBuffers(n: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut buffer: GLuint = 0;
        bindings::glGenBuffers(n, &mut buffer as _);
        vec![buffer]
    }
}

pub fn glGenTextures(n: GLsizei) -> Vec<GLuint> {
    unsafe {
        let mut texture: GLuint = 0;
        bindings::glGenTextures(n, &mut texture as _);
        vec![texture]
    }
}

pub fn glGetUniformLocation(program: GLuint, name: &str) -> GLint {
    unsafe {
        let name = CString::new(name).unwrap_or_else(|e| panic!("{}", e));
        bindings::glGetUniformLocation(program, name.as_ptr() as *const _)
    }
}

pub fn glShaderSource(shader: GLuint, source: &str) {
    unsafe {
        let source = CString::new(source).unwrap_or_else(|e| panic!("{}", e));
        bindings::glShaderSource(shader, 1, [source].as_ptr() as *const _, std::ptr::null());
    }
}

pub fn glGetAttribLocation(program: GLuint, name: &str) -> GLint {
    unsafe {
        let name = CString::new(name).unwrap_or_else(|e| panic!("{}", e));
        bindings::glGetAttribLocation(program, name.as_ptr() as *const _)
    }
}

pub fn glTexImage2DByBytes(
    target: GLenum,
    level: GLint,
    internal_format: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    r#type: GLenum,
    data: &[u8],
) {
    unsafe {
        bindings::glTexImage2D(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            r#type,
            data.as_ptr() as *const _,
        );
    }
}

pub fn glTexImage2DByPointer(
    _target: GLenum,
    _level: GLint,
    _internal_format: GLint,
    _width: GLsizei,
    _height: GLsizei,
    _border: GLint,
    _format: GLenum,
    _type: GLenum,
    _ptr: GLuint,
) {
    panic!("glTexImage2DPointer has no binding for opengl")
}

pub fn glGetShaderParameter(shader: GLuint, pname: GLenum) -> GLint {
    unsafe {
        let mut ret: GLint = 0;
        bindings::glGetShaderiv(shader, pname, &mut ret);
        ret
    }
}

pub fn glGetShaderInfoLog(shader: GLuint) -> String {
    let mut length = glGetShaderParameter(shader, GL_INFO_LOG_LENGTH);
    let mut messages = vec![0u8; length as usize + 1];
    unsafe {
        bindings::glGetShaderInfoLog(
            shader,
            length,
            &mut length as *mut _,
            messages.as_mut_ptr() as *mut _,
        );
        let messages = std::string::String::from_utf8_lossy(&messages[0..length as usize - 1]);
        messages.to_string()
    }
}

pub fn glGetProgramParameter(program: GLuint, pname: GLenum) -> GLint {
    unsafe {
        let mut ret: GLint = 0;
        bindings::glGetProgramiv(program, pname, &mut ret);
        ret
    }
}

pub fn glGetProgramInfoLog(program: GLuint) -> String {
    let mut length = glGetProgramParameter(program, GL_INFO_LOG_LENGTH);
    let mut messages = vec![0u8; length as usize + 1];
    unsafe {
        bindings::glGetProgramInfoLog(
            program,
            length,
            &mut length as *mut _,
            messages.as_mut_ptr() as *mut _,
        );
        let messages = std::string::String::from_utf8_lossy(&messages[0..length as usize - 1]);
        messages.to_string()
    }
}
