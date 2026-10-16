use xs_backend_wayland_client::{
    WL_DISPLAY_GET_REGISTRY, wl_output, wl_output_listener, wl_proxy, wl_registry,
    wl_registry_listener, wl_request_constructor,
};
use xs_backend_wayland_extensions::xdg_shell::xdg_wm_base_interface;

use crate::context::WaylandContext;

/// `wl_output.scale`, in device pixels per surface-local pixel.
///
/// A fallback for `wl_surface.preferred_buffer_scale`, which is the one that
/// counts: `WaylandContext::apply_scale` drops this reading once the surface
/// hint has spoken. It arrives as its own event rather than as an argument to
/// `done`, so a listener only sees it on a `wl_output` bound at version 2 or
/// later, and it is optional even then - its own documentation says a client
/// that never receives it should assume 1, and the compositor this was developed
/// against advertises version 4 and sends none.
pub unsafe extern "C" fn wl_output_handle_scale(
    data: *mut std::ffi::c_void,
    _output: *mut wl_output,
    factor: i32,
) {
    unsafe {
        assert!(!data.is_null());
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        ctx.apply_scale(factor as f32, false);
    }
}

/// Everything else about the output - geometry, mode, enter and leave - is not
/// something this backend acts on yet. A surface on more than one output, each
/// with a different scale, would need the `enter`/`leave` pair to pick the right
/// one; until then the first scale to arrive is the one the whole app uses.
static WL_OUTPUT_LISTENER: wl_output_listener = {
    let mut listener = wl_output_listener::dummy();
    listener.scale = wl_output_handle_scale;
    listener
};

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
                // Six is the floor that matters twice over:
                // `wl_surface` inherits the compositor's version, and
                // `set_buffer_scale` is a `wl_surface` request since version 3
                // while `preferred_buffer_scale` is an event since version 6.
                // Asking for one raises a protocol error - the first version
                // shows up as the app coming up with nothing drawn at all - so
                // the version is worth being explicit about.
                let compositor_version = 6.min(version);
                ctx.wl_compositor = ctx.client.wl_registry_bind(
                    registry,
                    id,
                    ctx.client.wl_compositor_interface,
                    compositor_version,
                ) as _;
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
            "wl_output" => {
                // Two is the first version that reports a scale. Binding higher
                // only adds `name` and `description`, which nothing reads.
                let output_version = 2.min(version);
                ctx.wl_output = ctx.client.wl_registry_bind(
                    registry,
                    id,
                    ctx.client.wl_output_interface,
                    output_version,
                ) as _;
                (ctx.client.wl_proxy_add_listener)(
                    ctx.wl_output as _,
                    &raw const WL_OUTPUT_LISTENER as *const _ as _,
                    ctx as *mut _ as _,
                );
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
