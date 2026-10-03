#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::{
    GLvoid,
    bindings::{
        self,
        common_sys::{GLboolean, GLenum, GLint, GLintptr, GLsizei, GLuint},
    },
};

pub fn glBufferData<T>(target: GLenum, data: &[T], usage: GLenum) {
    unsafe {
        bindings::glBufferData(
            target,
            std::mem::size_of_val(data) as _,
            data.as_ptr() as *const GLvoid,
            usage,
        );
    }
}

pub fn glVertexAttribPointer(
    index: GLuint,
    size: GLint,
    type_: GLenum,
    normalized: GLboolean,
    stride: GLsizei,
    offset: GLintptr,
) {
    unsafe {
        bindings::glVertexAttribPointer(index, size, type_, normalized, stride, offset);
    }
}

pub fn glGenBuffers(_n: GLsizei) -> Vec<GLuint> {
    unsafe {
        let buffer = bindings::glCreateBuffer();
        vec![buffer]
    }
}

pub fn glGenTextures(_n: GLsizei) -> Vec<GLuint> {
    unsafe {
        let texture = bindings::glCreateTexture();
        vec![texture]
    }
}

pub fn glGetUniformLocation(program: GLuint, name: &str) -> GLint {
    unsafe {
        let ptr = name.as_ptr();
        let len = name.len();
        bindings::glGetUniformLocation(program, ptr as _, len as _)
    }
}

pub fn glShaderSource(shader: GLuint, source: &str) {
    unsafe {
        let ptr = source.as_ptr();
        let len = source.len();
        bindings::glShaderSource(shader, ptr as _, len as _);
    }
}

pub fn glGetAttribLocation(program: GLuint, name: &str) -> GLint {
    unsafe {
        let ptr = name.as_ptr();
        let len = name.len();
        bindings::glGetAttribLocation(program, ptr as _, len as _)
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
        let ptr_data = data.as_ptr();
        let len_data = data.len();
        bindings::glTexImage2DByBytes(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            r#type,
            ptr_data,
            len_data as _,
        );
    }
}

pub fn glTexImage2DByPointer(
    target: GLenum,
    level: GLint,
    internal_format: GLint,
    width: GLsizei,
    height: GLsizei,
    border: GLint,
    format: GLenum,
    r#type: GLenum,
    source: GLint,
) {
    unsafe {
        bindings::glTexImage2DByPointer(
            target,
            level,
            internal_format,
            width,
            height,
            border,
            format,
            r#type,
            source,
        );
    }
}

pub fn glGetShaderParameter(shader: GLuint, pname: GLenum) -> GLint {
    unsafe { bindings::glGetShaderParameter(shader, pname) }
}

pub fn glGetShaderInfoLog(shader: GLuint) -> String {
    unsafe {
        let info_log = bindings::glGetShaderInfoLog(shader);
        std::ffi::CStr::from_ptr(info_log as _)
            .to_str()
            .unwrap()
            .to_string()
    }
}

pub fn glGetProgramParameter(program: GLuint, pname: GLenum) -> GLint {
    unsafe { bindings::glGetProgramParameter(program, pname) }
}

pub fn glGetProgramInfoLog(program: GLuint) -> String {
    unsafe {
        let info_log = bindings::glGetProgramInfoLog(program);
        std::ffi::CStr::from_ptr(info_log as _)
            .to_str()
            .unwrap()
            .to_string()
    }
}
