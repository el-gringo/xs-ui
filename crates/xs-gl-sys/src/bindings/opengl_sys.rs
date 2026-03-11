#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    dead_code
)]

use crate::bindings::common_sys::*;

macro_rules! gl_loader {
    (
        $(
            fn $fn:ident ( $($arg:ident : $t:ty),* ) -> $res:ty
        ),*
    ) => {
        mod __pfns {
            use super::*;

            $(
                pub static mut $fn: Option<extern "C" fn ($($arg: $t),*) -> $res> = None;
            )*
        }

        $(
            pub unsafe fn $fn($($arg: $t),*) -> $res {
                unsafe {
                    __pfns::$fn.unwrap()( $($arg),* )
                }
            }
        )*

        pub fn load_gl_funcs<T: FnMut(&str) -> Option<unsafe extern "C" fn() -> ()>>(mut getprocaddr: T) {
            $(
                unsafe {
                    let fn_name = stringify!($fn);
                    let addr = getprocaddr(fn_name);
                    __pfns::$fn = ::std::mem::transmute_copy(&addr);
                }
            )*
        }
    };
}

gl_loader!(
    fn glGetStringi(name: GLenum, index: GLuint) -> *const GLubyte,
    fn glGetString(name: GLenum) -> *const GLubyte,
    fn glFramebufferTextureLayer(
        target: GLenum,
        attachment: GLenum,
        texture: GLuint,
        level: GLint,
        layer: GLint
    ) -> (),
    fn glGenFramebuffers(n: GLsizei, framebuffers: *mut GLuint) -> (),
    fn glBindFramebuffer(target: GLenum, framebuffer: GLuint) -> (),
    fn glBindRenderbuffer(target: GLenum, renderbuffer: GLuint) -> (),
    fn glClearBufferfi(buffer: GLenum, drawbuffer: GLint, depth: GLfloat, stencil: GLint) -> (),
    fn glClearBufferfv(buffer: GLenum, drawbuffer: GLint, value: *const GLfloat) -> (),
    fn glClearBufferuiv(buffer: GLenum, drawbuffer: GLint, value: *const GLuint) -> (),
    fn glDeleteRenderbuffers(n: GLsizei, renderbuffers: *const GLuint) -> (),
    fn glUniform4fv(location: GLint, count: GLsizei, value: *const GLfloat) -> (),
    fn glUniform3fv(location: GLint, count: GLsizei, value: *const GLfloat) -> (),
    fn glUniform2fv(location: GLint, count: GLsizei, value: *const GLfloat) -> (),
    fn glUniform1fv(location: GLint, count: GLsizei, value: *const GLfloat) -> (),
    fn glUniform1iv(location: GLint, count: GLsizei, value: *const GLint) -> (),
    fn glUniform2iv(location: GLint, count: GLsizei, value: *const GLint) -> (),
    fn glUniform3iv(location: GLint, count: GLsizei, value: *const GLint) -> (),
    fn glUniform4iv(location: GLint, count: GLsizei, value: *const GLint) -> (),
    fn glUniform1i(location: GLint, v0: GLint) -> (),
    fn glUniform2i(location: GLint, v0: GLint, v1: GLint) -> (),
    fn glUniform3i(location: GLint, v0: GLint, v1: GLint, v2: GLint) -> (),
    fn glUniform4i(location: GLint, v0: GLint, v1: GLint, v2: GLint, v3: GLint) -> (),
    fn glUniform1f(location: GLint, v0: GLfloat) -> (),
    fn glUniform2f(location: GLint, v0: GLfloat, v1: GLfloat) -> (),
    fn glUniform3f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat) -> (),
    fn glUniform4f(location: GLint, v0: GLfloat, v1: GLfloat, v2: GLfloat, v3: GLfloat) -> (),
    fn glUseProgram(program: GLuint) -> (),
    fn glShaderSource(
        shader: GLuint,
        count: GLsizei,
        string: *const *const GLchar,
        length: *const GLint
    ) -> (),
    fn glLinkProgram(program: GLuint) -> (),
    fn glPixelStorei(pname: GLenum, param: GLint) -> (),
    fn glGetUniformLocation(program: GLuint, name: *const GLchar) -> GLint,
    fn glGetShaderiv(shader: GLuint, pname: GLenum, params: *mut GLint) -> (),
    fn glGetProgramInfoLog(
        program: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        infoLog: *mut GLchar
    ) -> (),
    fn glGetAttribLocation(program: GLuint, name: *const GLchar) -> GLint,
    fn glDisableVertexAttribArray(index: GLuint) -> (),
    fn glDeleteShader(shader: GLuint) -> (),
    fn glDeleteProgram(program: GLuint) -> (),
    fn glCompileShader(shader: GLuint) -> (),
    fn glStencilFuncSeparate(face: GLenum, func: GLenum, ref_: GLint, mask: GLuint) -> (),
    fn glStencilOpSeparate(face: GLenum, sfail: GLenum, dpfail: GLenum, dppass: GLenum) -> (),
    fn glRenderbufferStorageMultisample(
        target: GLenum,
        samples: GLsizei,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei
    ) -> (),
    fn glDrawBuffers(n: GLsizei, bufs: *const GLenum) -> (),
    fn glVertexAttribDivisor(index: GLuint, divisor: GLuint) -> (),
    fn glBufferSubData(
        target: GLenum,
        offset: GLintptr,
        size: GLsizeiptr,
        data: *const ::std::os::raw::c_void
    ) -> (),
    fn glGenBuffers(n: GLsizei, buffers: *mut GLuint) -> (),
    fn glCheckFramebufferStatus(target: GLenum) -> GLenum,
    fn glFramebufferRenderbuffer(
        target: GLenum,
        attachment: GLenum,
        renderbuffertarget: GLenum,
        renderbuffer: GLuint
    ) -> (),
    fn glCompressedTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        imageSize: GLsizei,
        data: *const GLvoid
    ) -> (),
    fn glCompressedTexImage3D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        border: GLint,
        imageSize: GLsizei,
        data: *const GLvoid
    ) -> (),
    fn glActiveTexture(texture: GLenum) -> (),
    fn glTexSubImage3D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid
    ) -> (),
    fn glUniformMatrix2fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat
    ) -> (),
    fn glUniformMatrix3fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat
    ) -> (),
    fn glUniformMatrix4fv(
        location: GLint,
        count: GLsizei,
        transpose: GLboolean,
        value: *const GLfloat
    ) -> (),
    fn glRenderbufferStorage(
        target: GLenum,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei
    ) -> (),
    fn glPolygonOffset(factor: GLfloat, units: GLfloat) -> (),
    fn glDrawElements(mode: GLenum, count: GLsizei, type_: GLenum, indices: *const GLvoid) -> (),
    fn glDeleteFramebuffers(n: GLsizei, framebuffers: *const GLuint) -> (),
    fn glBlendEquationSeparate(modeRGB: GLenum, modeAlpha: GLenum) -> (),
    fn glDeleteTextures(n: GLsizei, textures: *const GLuint) -> (),
    fn glGetProgramiv(program: GLuint, pname: GLenum, params: *mut GLint) -> (),
    fn glBindTexture(target: GLenum, texture: GLuint) -> (),
    fn glTexImage3D(
        target: GLenum,
        level: GLint,
        internalFormat: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid
    ) -> (),
    fn glCreateShader(type_: GLenum) -> GLuint,
    fn glTexSubImage2D(
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid
    ) -> (),
    fn glCopyTexImage2D(
        target: GLenum,
        level: GLint,
        internalformat: GLenum,
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint
    ) -> (),
    fn glClearDepthf(d: GLfloat) -> (),
    fn glClearDepth(depth: GLclampd) -> (),
    fn glFramebufferTexture2D(
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint
    ) -> (),
    fn glCreateProgram() -> GLuint,
    fn glViewport(x: GLint, y: GLint, width: GLsizei, height: GLsizei) -> (),
    fn glDeleteBuffers(n: GLsizei, buffers: *const GLuint) -> (),
    fn glDrawArrays(mode: GLenum, first: GLint, count: GLsizei) -> (),
    fn glDrawElementsInstanced(
        mode: GLenum,
        count: GLsizei,
        type_: GLenum,
        indices: *const ::std::os::raw::c_void,
        instancecount: GLsizei
    ) -> (),
    fn glVertexAttribPointer(
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: GLboolean,
        stride: GLsizei,
        pointer: *const ::std::os::raw::c_void
    ) -> (),
    fn glVertexAttribIPointer(
        index: GLuint,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        pointer: *const ::std::os::raw::c_void
    ) -> (),
    fn glDisable(cap: GLenum) -> (),
    fn glColorMask(red: GLboolean, green: GLboolean, blue: GLboolean, alpha: GLboolean) -> (),
    fn glBindBuffer(target: GLenum, buffer: GLuint) -> (),
    fn glBindVertexArray(array: GLuint) -> (),
    fn glDeleteVertexArrays(n: GLsizei, arrays: *const GLuint) -> (),
    fn glDepthMask(flag: GLboolean) -> (),
    fn glDrawArraysInstanced(
        mode: GLenum,
        first: GLint,
        count: GLsizei,
        instancecount: GLsizei
    ) -> (),
    fn glClearStencil(s: GLint) -> (),
    fn glScissor(x: GLint, y: GLint, width: GLsizei, height: GLsizei) -> (),
    fn glGenRenderbuffers(n: GLsizei, renderbuffers: *mut GLuint) -> (),
    fn glBufferData(
        target: GLenum,
        size: GLsizeiptr,
        data: *const ::std::os::raw::c_void,
        usage: GLenum
    ) -> (),
    fn glBlendFuncSeparate(
        sfactorRGB: GLenum,
        dfactorRGB: GLenum,
        sfactorAlpha: GLenum,
        dfactorAlpha: GLenum
    ) -> (),
    fn glGenerateMipmap(target: GLenum) -> (),
    fn glTexParameteri(target: GLenum, pname: GLenum, param: GLint) -> (),
    fn glGetIntegerv(pname: GLenum, params: *mut GLint) -> (),
    fn glEnable(cap: GLenum) -> (),
    fn glBlitFramebuffer(
        srcX0: GLint,
        srcY0: GLint,
        srcX1: GLint,
        srcY1: GLint,
        dstX0: GLint,
        dstY0: GLint,
        dstX1: GLint,
        dstY1: GLint,
        mask: GLbitfield,
        filter: GLenum
    ) -> (),
    fn glStencilMask(mask: GLuint) -> (),
    fn glStencilMaskSeparate(face: GLenum, mask: GLuint) -> (),
    fn glAttachShader(program: GLuint, shader: GLuint) -> (),
    fn glDetachShader(program: GLuint, shader: GLuint) -> (),
    fn glGetError() -> GLenum,
    fn glClearColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf) -> (),
    fn glBlendColor(red: GLclampf, green: GLclampf, blue: GLclampf, alpha: GLclampf) -> (),
    fn glTexParameterf(target: GLenum, pname: GLenum, param: GLfloat) -> (),
    fn glTexParameterfv(target: GLenum, pname: GLenum, params: *const GLfloat) -> (),
    fn glTexParameteriv(target: GLenum, pname: GLenum, params: *const GLint) -> (),
    fn glGetShaderInfoLog(
        shader: GLuint,
        bufSize: GLsizei,
        length: *mut GLsizei,
        infoLog: *mut GLchar
    ) -> (),
    fn glDepthFunc(func: GLenum) -> (),
    fn glStencilOp(fail: GLenum, zfail: GLenum, zpass: GLenum) -> (),
    fn glStencilFunc(func: GLenum, ref_: GLint, mask: GLuint) -> (),
    fn glEnableVertexAttribArray(index: GLuint) -> (),
    fn glBlendFunc(sfactor: GLenum, dfactor: GLenum) -> (),
    fn glReadBuffer(mode: GLenum) -> (),
    fn glClear(mask: GLbitfield) -> (),
    fn glTexImage2D(
        target: GLenum,
        level: GLint,
        internalFormat: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        type_: GLenum,
        pixels: *const GLvoid
    ) -> (),
    fn glGenVertexArrays(n: GLsizei, arrays: *mut GLuint) -> (),
    fn glFrontFace(mode: GLenum) -> (),
    fn glCullFace(mode: GLenum) -> (),
    fn glGenTextures(n: GLsizei, textures: *mut GLuint) -> (),
    fn glReadPixels(
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        type_: GLenum,
        pixels: *mut GLvoid
    ) -> (),
    fn glBegin(target: GLenum) -> (),
    fn glBeginQuery(target: GLenum, id: GLuint) -> (),
    fn glDeleteQueries(n: GLsizei, ids: *const GLuint) -> (),
    fn glEnd() -> (),
    fn glEndQuery(target: GLenum) -> (),
    fn glGenQueries(n: GLsizei, ids: *mut GLuint) -> (),
    fn glGetQueryObjectiv(id: GLuint, pname: GLenum, params: *mut GLint) -> (),
    fn glGetQueryObjectui64v(id: GLuint, pname: GLenum, params: *mut GLuint64) -> (),
    fn glVertex2i(x: GLfloat, y: GLfloat) -> (),

    fn glFlush() -> (),
    fn glFinish() -> (),
    fn glPolygonMode(face: GLenum, mode: GLenum) -> (),

    fn glVertex3f(x: GLfloat, y: GLfloat, z: GLfloat) -> (),
    fn glColor3f(r: GLfloat, g: GLfloat, b: GLfloat) -> (),
    fn glOrtho(g: GLdouble, d: GLdouble, b: GLdouble, h: GLdouble, cmin: GLdouble, cmax: GLdouble) -> (),
    fn glLoadIdentity() -> (),
    fn glMatrixMode(mode: GLenum) -> ()
);
