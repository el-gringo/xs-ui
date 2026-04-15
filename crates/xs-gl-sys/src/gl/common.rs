#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals)]

use crate::bindings;
pub use crate::bindings::common_sys::*;

pub fn glActiveTexture(texture: GLuint) {
    unsafe {
        bindings::glActiveTexture(texture);
    }
}
pub fn glClear(mask: GLbitfield) {
    unsafe {
        bindings::glClear(mask);
    }
}

pub fn glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat) {
    unsafe {
        bindings::glClearColor(red, green, blue, alpha);
    }
}

pub fn glColorMask(red: bool, green: bool, blue: bool, alpha: bool) {
    unsafe {
        bindings::glColorMask(red as _, green as _, blue as _, alpha as _);
    }
}

pub fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) {
    unsafe {
        bindings::glViewport(x, y, width, height);
    }
}

pub fn glBlendFunc(sfactor: GLenum, dfactor: GLenum) {
    unsafe {
        bindings::glBlendFunc(sfactor, dfactor);
    }
}

pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei) {
    unsafe {
        bindings::glDrawArrays(mode, first, count);
    }
}

pub fn glDrawElements(mode: GLenum, count: usize, r#type: GLenum, indices_byte_offset: usize) {
    unsafe {
        bindings::glDrawElements(
            mode,
            count as _,
            r#type as _,
            indices_byte_offset as *const _,
            0,
        );
    }
}

pub fn glEnableVertexAttribArray(index: GLuint) {
    unsafe {
        bindings::glEnableVertexAttribArray(index);
    }
}

pub fn glEnable(cap: GLenum) {
    unsafe {
        bindings::glEnable(cap);
    }
}

pub fn glDisable(cap: GLenum) {
    unsafe {
        bindings::glDisable(cap);
    }
}

pub fn glBindBuffer(target: GLenum, buffer: GLuint) {
    unsafe {
        bindings::glBindBuffer(target, buffer);
    }
}

pub fn glUniform1f(location: GLint, v0: GLfloat) {
    unsafe {
        bindings::glUniform1f(location, v0);
    }
}

pub fn glUniform1i(location: GLint, v0: GLint) {
    unsafe {
        bindings::glUniform1i(location, v0);
    }
}

pub fn glUniform2f(location: GLint, v0: GLfloat, v1: GLfloat) {
    unsafe {
        bindings::glUniform2f(location, v0, v1);
    }
}

pub fn glUniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat) {
    unsafe {
        bindings::glUniform3f(location, v0, v1, v2);
    }
}

pub fn glUniform4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat) {
    unsafe {
        bindings::glUniform4f(location, v0, v1, v2, v3);
    }
}

pub fn glUniform4fv(location: i32, value: &[f32; 4]) {
    unsafe {
        bindings::glUniform4fv(location, 1, value.as_ptr());
    }
}

pub fn glUniformMatrix3fv(location: i32, transpose: bool, value: &[f32; 9]) {
    unsafe {
        bindings::glUniformMatrix3fv(location, 1, transpose as _, value.as_ptr());
    }
}

pub fn glUniformMatrix4fv(location: GLint, transpose: bool, value: &[f32; 16]) {
    unsafe {
        bindings::glUniformMatrix4fv(location, 1, transpose as _, value.as_ptr());
    }
}

pub fn glUseProgram(program: GLuint) {
    unsafe {
        bindings::glUseProgram(program);
    }
}

pub fn glCreateProgram() -> GLuint {
    unsafe { bindings::glCreateProgram() }
}

pub fn glAttachShader(program: GLuint, shader: GLuint) {
    unsafe {
        bindings::glAttachShader(program, shader);
    }
}

pub fn glLinkProgram(program: GLuint) {
    unsafe {
        bindings::glLinkProgram(program);
    }
}

pub fn glCreateShader(shader_type: GLenum) -> GLuint {
    unsafe { bindings::glCreateShader(shader_type) }
}

pub fn glCompileShader(shader: GLuint) {
    unsafe {
        bindings::glCompileShader(shader);
    }
}

pub fn glTexParameteri(target: GLenum, param: GLuint, pname: GLint) {
    unsafe {
        bindings::glTexParameteri(target, param, pname);
    }
}

pub fn glGenerateMipmap(target: GLenum) {
    unsafe {
        bindings::glGenerateMipmap(target);
    }
}

pub fn glBindTexture(target: GLenum, texture: GLuint) {
    unsafe {
        bindings::glBindTexture(target, texture);
    }
}

pub fn glFlush() {
    unsafe {
        bindings::glFlush();
    }
}

pub fn glPixelStorei(pname: GLenum, param: GLint) {
    unsafe {
        bindings::glPixelStorei(pname, param);
    }
}
