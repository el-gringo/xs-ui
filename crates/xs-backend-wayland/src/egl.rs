use std::ptr::null_mut;

use xs_egl_sys::{
    EGL_ALPHA_SIZE, EGL_BLUE_SIZE, EGL_BUFFER_SIZE, EGL_CONTEXT_CLIENT_VERSION, EGL_DEPTH_SIZE,
    EGL_GREEN_SIZE, EGL_NONE, EGL_OPENGL_ES2_BIT, EGL_RED_SIZE, EGL_RENDERABLE_TYPE, EGLConfig,
    EGLContext, EGLDisplay, LibEgl,
};
use xs_gl_sys::load_gl_funcs;

use crate::context::WaylandContext;

pub unsafe fn create_egl_config(
    lib_egl: &mut LibEgl,
    display: EGLDisplay,
    alpha: bool,
) -> EGLConfig {
    let alpha_size = if alpha { 8 } else { 0 };
    #[rustfmt::skip]
    let attrib_list = [
        EGL_BUFFER_SIZE, 16,
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES2_BIT,
        EGL_NONE,
    ];
    let mut configs: Vec<EGLConfig> = vec![null_mut(); 32];
    let mut num_config = 0;
    unsafe {
        let _ret = (lib_egl.eglChooseConfig)(
            display,
            attrib_list.as_ptr() as _,
            configs.as_ptr() as _,
            32,
            &mut num_config as *mut _ as *mut _,
        );
    }
    assert!(num_config > 0);
    assert!(num_config <= 32);

    // find config with 8-bit rgb buffer if available, ndk sample does not trust egl spec
    let mut egl_config: EGLConfig = null_mut();
    let mut exact_cfg_found = false;
    for c in &mut configs[0..num_config] {
        let mut r: i32 = 0;
        let mut g: i32 = 0;
        let mut b: i32 = 0;
        let mut a: i32 = 0;
        let mut d: i32 = 0;
        unsafe {
            if (lib_egl.eglGetConfigAttrib)(display as _, *c, EGL_RED_SIZE as _, &mut r) == 1
                && (lib_egl.eglGetConfigAttrib)(display as _, *c, EGL_GREEN_SIZE as _, &mut g) == 1
                && (lib_egl.eglGetConfigAttrib)(display as _, *c, EGL_BLUE_SIZE as _, &mut b) == 1
                && (lib_egl.eglGetConfigAttrib)(display as _, *c, EGL_ALPHA_SIZE as _, &mut a) == 1
                && (lib_egl.eglGetConfigAttrib)(display as _, *c, EGL_DEPTH_SIZE as _, &mut d) == 1
                && r == 8
                && g == 8
                && b == 8
                && (alpha_size == 0 || a == alpha_size as _)
                && d == 16
            {
                exact_cfg_found = true;
                egl_config = *c;
                break;
            }
        }
    }
    if !exact_cfg_found {
        egl_config = configs[0];
    }
    egl_config
}

pub unsafe fn create_egl_context(
    lib_egl: &mut LibEgl,
    egl_config: EGLConfig,
    egl_display: EGLDisplay,
) -> Result<EGLContext, &'static str> {
    let ctx_attributes = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
    unsafe {
        let egl_context = (lib_egl.eglCreateContext)(
            egl_display,
            egl_config,
            /* EGL_NO_CONTEXT */ null_mut(),
            ctx_attributes.as_ptr() as _,
        );
        if egl_context.is_null() {
            return Err("EGL_NO_CONTEXT");
        }
        Ok(egl_context)
    }
}

/// Creates the egl window, sized in **buffer** pixels.
///
/// The size comes from [`WaylandContext::buffer_size`] rather than from the
/// caller's arguments because that is the one that knows about the output's
/// scale: `wl_egl_window_create` wants the number of pixels the GPU rasterises
/// into, not the surface-local size the compositor talks about. The compositor's
/// own size replaces this as soon as the first `xdg_toplevel.configure` lands.
pub fn create_egl_window(ctx: &mut WaylandContext) {
    let (width, height) = ctx.buffer_size();
    unsafe {
        load_gl_funcs(|proc| match std::ffi::CString::new(proc) {
            Ok(name) => (ctx.lib_egl.eglGetProcAddress)(name.as_ptr() as _),
            _ => None,
        });

        ctx.egl_display = (ctx.lib_egl.eglGetDisplay)(ctx.wl_display as _);
        if ctx.egl_display.is_null() {
            panic!("EGL_NO_DISPLAY");
        }
        let ret = (ctx.lib_egl.eglInitialize)(ctx.egl_display, null_mut(), null_mut());
        if ret == 0 {
            panic!("EGL_NOT_INITIALIZED");
        }
        ctx.egl_config = create_egl_config(&mut ctx.lib_egl, ctx.egl_display, false);
        ctx.egl_window = (ctx.lib_wayland_egl.wl_egl_window_create)(ctx.wl_surface, width, height);
        ctx.egl_surface = (ctx.lib_egl.eglCreateWindowSurface)(
            ctx.egl_display,
            ctx.egl_config,
            ctx.egl_window as _,
            std::ptr::null_mut(),
        );
        if ctx.egl_surface.is_null()
        /* EGL_NO_SURFACE  */
        {
            panic!("EGL_NO_SURFACE error");
        }

        ctx.egl_context =
            create_egl_context(&mut ctx.lib_egl, ctx.egl_config, ctx.egl_display).unwrap();
        if (ctx.lib_egl.eglMakeCurrent)(
            ctx.egl_display,
            ctx.egl_surface,
            ctx.egl_surface,
            ctx.egl_context,
        ) == 0
        {
            panic!("eglMakeCurrent error");
        }
    }
}
