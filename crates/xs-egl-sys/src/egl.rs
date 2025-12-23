#![allow(non_camel_case_types, non_snake_case, dead_code)]

pub type EGLNativeDisplayType = *mut ();
pub type EGLNativePixmapType = ::core::ffi::c_ulong;
pub type EGLNativeWindowType = ::core::ffi::c_ulong;

pub use core::ptr::null_mut;

pub const EGL_SUCCESS: u32 = 12288;

pub const EGL_WINDOW_BIT: u32 = 4;

pub const EGL_ALPHA_SIZE: u32 = 12321;
pub const EGL_BLUE_SIZE: u32 = 12322;
pub const EGL_BUFFER_SIZE: u32 = 0x3020;
pub const EGL_GREEN_SIZE: u32 = 12323;
pub const EGL_RED_SIZE: u32 = 12324;
pub const EGL_DEPTH_SIZE: u32 = 12325;
pub const EGL_STENCIL_SIZE: u32 = 12326;
pub const EGL_SAMPLES: u32 = 12337;
pub const EGL_NATIVE_VISUAL_ID: u32 = 12334;
pub const EGL_WIDTH: u32 = 12375;
pub const EGL_HEIGHT: u32 = 12374;
pub const EGL_OPENGL_ES2_BIT: u32 = 0x0004;
pub const EGL_RENDERABLE_TYPE: u32 = 0x3040;
pub const EGL_SURFACE_TYPE: u32 = 12339;
pub const EGL_NONE: u32 = 12344;
pub const EGL_CONTEXT_CLIENT_VERSION: u32 = 12440;

pub type NativeDisplayType = EGLNativeDisplayType;
pub type NativePixmapType = EGLNativePixmapType;
pub type NativeWindowType = EGLNativeWindowType;
pub type EGLint = i32;
pub type EGLBoolean = ::core::ffi::c_uint;
pub type EGLDisplay = *mut ::core::ffi::c_void;
pub type EGLConfig = *mut ::core::ffi::c_void;
pub type EGLSurface = *mut ::core::ffi::c_void;
pub type EGLContext = *mut ::core::ffi::c_void;
pub type __eglMustCastToProperFunctionPointerType = ::std::option::Option<unsafe extern "C" fn()>;

xs_platform_linux::declare_module! {
    LibEgl,
    "libEGL.so",
    "libEGL.so.1",
    ...
    ...
    pub fn eglChooseConfig(
        EGLDisplay,
        *const EGLint,
        *mut EGLConfig,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglCopyBuffers(
        EGLDisplay,
        EGLSurface,
        EGLNativePixmapType,
    ) -> EGLBoolean,
    pub fn eglCreateContext(
        EGLDisplay,
        EGLConfig,
        EGLContext,
        *const EGLint,
    ) -> EGLContext,
    pub fn eglCreatePbufferSurface(
        EGLDisplay,
        EGLConfig,
        *const EGLint,
    ) -> EGLSurface,
    pub fn eglCreatePixmapSurface(
        EGLDisplay,
        EGLConfig,
        EGLNativePixmapType,
        *const EGLint,
    ) -> EGLSurface,
    pub fn eglCreateWindowSurface(
        EGLDisplay,
        EGLConfig,
        EGLNativeWindowType,
        *const EGLint,
    ) -> EGLSurface,
    pub fn eglDestroyContext(EGLDisplay, EGLContext) -> EGLBoolean,
    pub fn eglDestroySurface(EGLDisplay, EGLSurface) -> EGLBoolean,
    pub fn eglGetConfigAttrib(
        EGLDisplay,
        EGLConfig,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglGetConfigs(
        EGLDisplay,
        *mut EGLConfig,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglGetCurrentDisplay() -> EGLDisplay,
    pub fn eglGetCurrentSurface(EGLint) -> EGLSurface,
    pub fn eglGetDisplay(EGLNativeDisplayType) -> EGLDisplay,
    pub fn eglGetError() -> EGLint,
    pub fn eglGetProcAddress(
        *const ::core::ffi::c_char,
    ) -> __eglMustCastToProperFunctionPointerType,
    pub fn eglInitialize(EGLDisplay, *mut EGLint, *mut EGLint) -> EGLBoolean,
    pub fn eglMakeCurrent(
        EGLDisplay,
        EGLSurface,
        EGLSurface,
        EGLContext,
    ) -> EGLBoolean,
    pub fn eglQueryContext(
        EGLDisplay,
        EGLContext,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglQueryString(EGLDisplay, EGLint) -> *const ::core::ffi::c_char,
    pub fn eglQuerySurface(
        EGLDisplay,
        EGLSurface,
        EGLint,
        *mut EGLint,
    ) -> EGLBoolean,
    pub fn eglSwapBuffers(EGLDisplay, EGLSurface) -> EGLBoolean,
    pub fn eglTerminate(EGLDisplay) -> EGLBoolean,
    pub fn eglWaitGL() -> EGLBoolean,
    pub fn eglWaitNative(EGLint) -> EGLBoolean,
    pub fn eglBindTexImage(EGLDisplay, EGLSurface, EGLint) -> EGLBoolean,
    pub fn eglReleaseTexImage(EGLDisplay, EGLSurface, EGLint) -> EGLBoolean,
    pub fn eglSurfaceAttrib(
        EGLDisplay,
        EGLSurface,
        EGLint,
        EGLint,
    ) -> EGLBoolean,
    pub fn eglSwapInterval(EGLDisplay, EGLint) -> EGLBoolean,
    ...
    ...
}
