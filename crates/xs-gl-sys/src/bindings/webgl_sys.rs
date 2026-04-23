#![allow(
    non_snake_case,
    non_camel_case_types,
    non_upper_case_globals,
    //dead_code
)]

use crate::{
    GLbitfield, GLboolean, GLchar, GLenum, GLfloat, GLint, GLintptr, GLsizei, GLsizeiptr, GLubyte,
    GLuint, GLvoid,
};

#[link(wasm_import_module = "env")]
unsafe extern "C" {
    pub fn glActiveTexture(texture: GLuint);
    pub fn glClear(mask: GLbitfield);
    pub fn glClearColor(red: GLfloat, green: GLfloat, blue: GLfloat, alpha: GLfloat);
    pub fn glColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean);
    pub fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei);
    pub fn glBlendFunc(sfactor: GLenum, dfactor: GLenum);
    pub fn glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        offset: GLintptr,
    );
    pub fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei);
    pub fn glDrawElements(
        mode: GLenum,
        count: GLsizei,
        type_: GLenum,
        indices: *const GLvoid,
        indice_count: GLsizei,
    );
    pub fn glEnableVertexAttribArray(index: GLuint);
    pub fn glBufferData(
        target: GLenum,
        size: GLsizeiptr,
        data: *const ::core::ffi::c_void,
        usage: GLenum,
    );

    pub fn glEnable(cap: GLenum);
    pub fn glDisable(cap: GLenum);
    pub fn glBindBuffer(target: GLenum, buffer: GLuint);
    pub fn glUniform1i(location: GLint, v0: GLint);
    pub fn glUniform1f(location: GLint, v0: GLfloat);
    pub fn glUniform2f(location: GLint, v0: GLfloat, v1: GLfloat);
    pub fn glUniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat);
    pub fn glUniform4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat);
    pub fn glUniform4fv(location: GLint, count: GLsizei, value: *const GLfloat);
    pub fn glUniformMatrix3fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );
    pub fn glUniformMatrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat,
    );
    pub fn glUseProgram(program: GLuint);
    pub fn glGetUniformLocation(program: GLuint, name: *const GLchar, len: GLuint) -> GLint;
    pub fn glCreateBuffer() -> GLuint;
    pub fn glCreateProgram() -> GLuint;
    pub fn glAttachShader(program: GLuint, shader: GLuint);
    pub fn glLinkProgram(program: GLuint);
    pub fn glCreateShader(type_: GLenum) -> GLuint;
    pub fn glShaderSource(shader: GLuint, source: *const GLchar, length: GLsizei);
    pub fn glCompileShader(shader: GLuint);
    pub fn glGetAttribLocation(program: GLuint, name: *const GLchar, len: GLuint) -> GLint;
    pub fn glTexParameteri(target: GLenum, param: GLuint, pname: GLint);
    pub fn glTexImage2DByPointer(
        target: GLenum,
        level: GLint,
        internalFormat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        r#type: GLenum,
        source: GLint,
    );
    pub fn glTexImage2DByBytes(
        target: GLenum,
        level: GLint,
        internalFormat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        r#type: GLenum,
        ptr_data: *const u8,
        len_data: GLuint,
    );
    pub fn glGenerateMipmap(target: GLenum);
    pub fn glCreateTexture() -> GLuint;
    pub fn glBindTexture(target: GLenum, texture: GLuint);
    pub fn glFlush();
    pub fn glPixelStorei(pname: GLenum, param: GLint);
    pub fn glGetProgramParameter(program: GLuint, pname: GLenum) -> GLint;
    pub fn glGetProgramInfoLog(program: GLuint) -> *const GLubyte;
    pub fn glGetShaderParameter(shader: GLuint, pname: GLenum) -> GLint;
    pub fn glGetShaderInfoLog(shader: GLuint) -> *const GLubyte;
}
