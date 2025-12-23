use xs_backend_wayland_client::{
    wl_proxy, wl_registry, wl_registry_listener, wl_request_constructor, WL_DISPLAY_GET_REGISTRY,
};
use xs_backend_wayland_extensions::xdg_shell::xdg_wm_base_interface;

use crate::context::WaylandContext;

pub unsafe extern "C" fn registry_add_object(
    data: *mut std::ffi::c_void,
    registry: *mut wl_registry,
    id: u32,
    interface: *const ::std::os::raw::c_char,
    version: u32,
) {
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        let interface = std::ffi::CStr::from_ptr(interface).to_str().unwrap();
        match interface {
            "wl_compositor" => {
                ctx.wl_compositor =
                    ctx.client
                        .wl_registry_bind(registry, id, ctx.client.wl_compositor_interface, 1)
                        as _;
            }
            "xdg_wm_base" => {
                ctx.xdg_wm_base =
                    ctx.client
                        .wl_registry_bind(registry, id, &raw const xdg_wm_base_interface, 1)
                        as _;
            }
            "wl_seat" => {
                let seat_version = 4.min(version);
                ctx.wl_seat = ctx.client.wl_registry_bind(
                    registry,
                    id,
                    ctx.client.wl_seat_interface,
                    seat_version,
                ) as _;
            }
            _ => {}
        }
    }
}

pub fn create_registry(ctx: &mut WaylandContext) {
    unsafe {
        ctx.wl_registry = wl_request_constructor!(
            ctx.client,
            ctx.wl_display,
            WL_DISPLAY_GET_REGISTRY,
            ctx.client.wl_registry_interface
        );
        let mut registry_listener = wl_registry_listener::dummy();
        registry_listener.global = registry_add_object;
        (ctx.client.wl_proxy_add_listener)(
            ctx.wl_registry as _,
            &registry_listener as *const _ as _,
            ctx as *mut _ as _,
        );
        (ctx.client.wl_display_roundtrip)(ctx.wl_display);
    }
}
