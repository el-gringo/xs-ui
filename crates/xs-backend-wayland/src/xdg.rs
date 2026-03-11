use xs_backend_wayland_client::{wl_array, wl_proxy, wl_request, wl_request_constructor};
use xs_backend_wayland_extensions::xdg_shell::{
    self, xdg_surface_listener, xdg_toplevel_listener, xdg_wm_base_listener,
};
use xs_gl_sys::{glClear, glClearColor, glFlush, glViewport, GL_COLOR_BUFFER_BIT};

use crate::context::WaylandContext;

pub unsafe extern "C" fn xdg_wm_base_handle_ping(
    data: *mut std::ffi::c_void,
    _toplevel: *mut xdg_shell::xdg_wm_base,
    serial: u32,
) -> () {
    unsafe {
        assert!(!data.is_null());
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        wl_request!(
            ctx.client,
            ctx.xdg_wm_base,
            xdg_shell::xdg_wm_base::pong,
            serial
        );
    }
}

unsafe extern "C" fn xdg_surface_handle_configure(
    data: *mut std::ffi::c_void,
    xdg_surface: *mut xdg_shell::xdg_surface,
    serial: u32,
) {
    assert!(!data.is_null());
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        if xdg_surface == ctx.xdg_surface {
            wl_request!(
                ctx.client,
                ctx.xdg_surface,
                xdg_shell::xdg_surface::ack_configure,
                serial
            );
            ctx.sufarce_commit();
        }
    }
}

unsafe extern "C" fn xdg_toplevel_handle_configure(
    data: *mut std::ffi::c_void,
    _xdg_toplevel: *mut xdg_shell::xdg_toplevel,
    width: i32,
    height: i32,
    _states: *mut wl_array,
) -> () {
    assert!(!data.is_null());
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        if width != 0 && height != 0 {
            (ctx.lib_wayland_egl.wl_egl_window_resize)(ctx.egl_window, width, height, 0, 0);
            glViewport(0, 0, width, height);
            glClearColor(0., 0., 0., 0.);
            glClear(GL_COLOR_BUFFER_BIT);
            glFlush();
            ctx.swap_buffers();
            ctx.sufarce_commit();
        }
    }
}

unsafe extern "C" fn xdg_toplevel_handle_close(
    data: *mut std::ffi::c_void,
    _xdg_toplevel: *mut xdg_shell::xdg_toplevel,
) {
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        ctx.app_context.running = false;
    }
}

pub static mut XDG_WM_BASE_LISTENER: xdg_wm_base_listener = xdg_wm_base_listener {
    ping: xdg_wm_base_handle_ping,
};

pub static mut XDG_SURFACE_LISTENER: xdg_surface_listener = xdg_surface_listener {
    configure: xdg_surface_handle_configure,
};

pub static mut XDG_TOPLEVEL_LISTENER: xdg_toplevel_listener = xdg_toplevel_listener {
    configure: xdg_toplevel_handle_configure,
    close: xdg_toplevel_handle_close,
};

pub fn create_xdg(ctx: &mut WaylandContext) {
    unsafe {
        (ctx.client.wl_proxy_add_listener)(
            ctx.xdg_wm_base as _,
            &raw const XDG_WM_BASE_LISTENER as *const _ as _,
            ctx as *mut _ as _,
        );

        ctx.xdg_surface = wl_request_constructor!(
            ctx.client,
            ctx.xdg_wm_base,
            xdg_shell::xdg_wm_base::get_xdg_surface,
            &raw const xdg_shell::xdg_surface_interface,
            ctx.wl_surface
        );
        assert!(ctx.xdg_surface.is_null() == false);

        (ctx.client.wl_proxy_add_listener)(
            ctx.xdg_surface as _,
            &raw const XDG_SURFACE_LISTENER as *const _ as _,
            ctx as *mut _ as _,
        );

        ctx.xdg_toplevel = wl_request_constructor!(
            ctx.client,
            ctx.xdg_surface,
            xdg_shell::xdg_surface::get_toplevel,
            &raw const xdg_shell::xdg_toplevel_interface
        );
        assert!(!ctx.xdg_toplevel.is_null());
        (ctx.client.wl_proxy_add_listener)(
            ctx.xdg_toplevel as _,
            &raw const XDG_TOPLEVEL_LISTENER as *const _ as _,
            ctx as *mut _ as _,
        );

        ctx.round_trip();
    }
}

pub fn set_app_id(ctx: &mut WaylandContext, app_id: String) {
    let app_id = std::ffi::CString::new(app_id).unwrap();
    unsafe {
        wl_request!(
            ctx.client,
            ctx.xdg_toplevel,
            xdg_shell::xdg_toplevel::set_app_id,
            app_id.as_ptr()
        );
    }
}

pub fn set_title(ctx: &mut WaylandContext, title: String) {
    let title = std::ffi::CString::new(title).unwrap();
    unsafe {
        wl_request!(
            ctx.client,
            ctx.xdg_toplevel,
            xdg_shell::xdg_toplevel::set_title,
            title.as_ptr()
        );
    }
}
