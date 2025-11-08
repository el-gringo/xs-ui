#[allow(static_mut_refs)]
use xs_backend_wayland_client::{wl_proxy, wl_request, wl_request_constructor, wl_surface, LibWaylandClient};
use xs_backend_wayland_extensions::xdg_shell::{self, xdg_toplevel};

use crate::registry::XDG_WM_BASE;


unsafe extern "C" fn xdg_wm_base_handle_ping(
    data: *mut std::ffi::c_void,
    toplevel: *mut xdg_shell::xdg_wm_base,
    serial: u32,
) -> () {
    unsafe {
        assert!(!data.is_null());
        let wayland_client: &mut LibWaylandClient = &mut *(data as *mut _);

        wl_request!(
            wayland_client,
            toplevel,
            xdg_shell::xdg_wm_base::pong,
            serial
        );
    }
}

pub unsafe fn init(wayland_client: &mut LibWaylandClient, surface: *mut wl_surface) -> *mut xdg_toplevel {
    unsafe {
        let xdg_surface: *mut xdg_shell::xdg_surface = wl_request_constructor!(
            wayland_client,
            XDG_WM_BASE,
            xdg_shell::xdg_wm_base::get_xdg_surface,
            &raw const xdg_shell::xdg_surface_interface,
            surface
        );
        assert!(xdg_surface.is_null() == false);

        let xdg_wm_base_listener = xdg_shell::xdg_wm_base_listener {
            ping: Some(xdg_wm_base_handle_ping),
        };

        (wayland_client.wl_proxy_add_listener)(
            XDG_WM_BASE as _,
            &xdg_wm_base_listener as *const _ as _,
            wayland_client as *mut _ as _,
        );

        let xdg_toplevel: *mut xdg_toplevel = wl_request_constructor!(
            wayland_client,
            xdg_surface,
            xdg_shell::xdg_surface::get_toplevel,
            &raw const xdg_shell::xdg_toplevel_interface
        );
        xdg_toplevel
    }
}
