use std::{ffi::c_void, ptr::null_mut};

use xs_backend_core::AppContext;
use xs_backend_wayland_client::{
    LibWaylandClient, WL_SURFACE_COMMIT, wl_compositor, wl_display, wl_registry, wl_request,
    wl_seat, wl_surface,
};
use xs_backend_wayland_egl::{LibWaylandEgl, wl_egl_window};
use xs_backend_wayland_extensions::xdg_shell::{xdg_surface, xdg_toplevel, xdg_wm_base};
use xs_backend_wayland_xkb::{LibXkbCommon, XkbKeymap, xkb_context, xkb_state};
use xs_egl_sys::LibEgl;

pub struct WaylandContext {
    pub app_context: AppContext,

    pub client: LibWaylandClient,
    pub lib_wayland_egl: LibWaylandEgl,
    pub lib_egl: LibEgl,
    pub lib_xkb: LibXkbCommon,

    pub egl_display: *mut c_void,
    pub egl_window: *mut wl_egl_window,
    pub egl_surface: *mut c_void,
    pub egl_config: *mut c_void,
    pub egl_context: *mut c_void,

    pub wl_display: *mut wl_display,
    pub wl_surface: *mut wl_surface,
    pub wl_registry: *mut wl_registry,
    pub wl_seat: *mut wl_seat,
    pub wl_compositor: *mut wl_compositor,

    pub xdg_wm_base: *mut xdg_wm_base,
    pub xdg_toplevel: *mut xdg_toplevel,
    pub xdg_surface: *mut xdg_surface,

    pub xkb_context: *mut xkb_context,
    pub xkb_state: *mut xkb_state,
    pub xkb_keymap: XkbKeymap,
}

impl WaylandContext {
    pub fn new() -> WaylandContext {
        let client = LibWaylandClient::try_load().expect("failed to load libwayland-client.so");
        let lib_wayland_egl = LibWaylandEgl::try_load().expect("failed to load libwayland-egl.so");
        let lib_egl = LibEgl::try_load().expect("failed to load libEGL.so");
        let lib_xkb = LibXkbCommon::try_load().expect("failed to load libxkbcommon.so");
        let app_context = AppContext::new();

        WaylandContext {
            app_context,

            client,
            lib_wayland_egl,
            lib_egl,
            lib_xkb,

            egl_display: null_mut(),
            egl_window: null_mut(),
            egl_surface: null_mut(),
            egl_config: null_mut(),
            egl_context: null_mut(),

            wl_display: null_mut(),
            wl_compositor: null_mut(),
            wl_registry: null_mut(),
            wl_surface: null_mut(),
            wl_seat: null_mut(),

            xdg_wm_base: null_mut(),
            xdg_surface: null_mut(),
            xdg_toplevel: null_mut(),

            xkb_context: null_mut(),
            xkb_state: null_mut(),
            xkb_keymap: Default::default()
        }
    }

    pub fn round_trip(&mut self) {
        unsafe {
            (self.client.wl_display_roundtrip)(self.wl_display);
        }
    }

    pub fn sufarce_commit(&mut self) {
        unsafe {
            wl_request!(self.client, self.wl_surface, WL_SURFACE_COMMIT);
        }
    }

    pub fn dispatch_pending(&mut self) -> i32 {
        unsafe { (self.client.wl_display_dispatch_pending)(self.wl_display) }
    }

    pub fn swap_buffers(&mut self) {
        unsafe {
            (self.lib_egl.eglSwapBuffers)(self.egl_display, self.egl_surface);
        }
    }
}
