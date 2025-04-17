
use xs_backend_wayland_client::{
    LibWaylandClient, wl_compositor, wl_display, wl_registry, wl_seat, wl_subcompositor
};
use xs_backend_wayland_extensions::{
    viewporter::{wp_viewporter, wp_viewporter_interface},
    xdg_shell,
};

use crate::seat::SEAT_LISTENER;

pub static mut COMPOSITOR: *mut wl_compositor = std::ptr::null_mut();
pub static mut SUBCOMPOSITOR: *mut wl_subcompositor = std::ptr::null_mut();
pub static mut XDG_WM_BASE: *mut xdg_shell::xdg_wm_base = std::ptr::null_mut();
pub static mut VIEWPORTER: *mut wp_viewporter = std::ptr::null_mut();
pub static mut SEAT: *mut wl_seat = std::ptr::null_mut();


unsafe extern "C" fn registry_add_object(
    data: *mut std::ffi::c_void,
    registry: *mut wl_registry,
    name: u32,
    interface: *const ::std::os::raw::c_char,
    version: u32,
) {
    unsafe {
        let client: &mut LibWaylandClient = &mut *(data as *mut _);
        let interface = std::ffi::CStr::from_ptr(interface).to_str().unwrap();
        match interface {
            "wl_compositor" => {
                COMPOSITOR =
                    client.wl_registry_bind(registry, name, client.wl_compositor_interface, 1) as _;
            }
            "wl_subcompositor" => {
                SUBCOMPOSITOR =
                    client.wl_registry_bind(registry, name, client.wl_subcompositor_interface, 1)
                        as _;
            }
            "xdg_wm_base" => {
                XDG_WM_BASE =
                    client.wl_registry_bind(registry, name, &raw const xdg_shell::xdg_wm_base_interface, 1)
                        as _;
            }
            "wp_viewporter" => {
                VIEWPORTER =
                    client.wl_registry_bind(registry, name, &raw const wp_viewporter_interface, 1) as _;
            }
            "wl_seat" => {
                let seat_version = 4.min(version);
                SEAT =
                    client.wl_registry_bind(registry, name, client.wl_seat_interface, seat_version)
                        as _;
                (client.wl_proxy_add_listener)(SEAT as _, &raw const SEAT_LISTENER as *const _ as _, data);
            }

            _ => {}
        }
    }
}

unsafe extern "C" fn registry_remove_object(
    _data: *mut std::ffi::c_void,
    _registry: *mut wl_registry,
    _name: u32,
) {
}

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Copy, Clone)]
pub struct wl_registry_listener {
    pub global: ::std::option::Option<
        unsafe extern "C" fn(
            data: *mut ::std::os::raw::c_void,
            wl_registry: *mut wl_registry,
            name: u32,
            interface: *const ::std::os::raw::c_char,
            version: u32,
        ),
    >,
    pub global_remove: ::std::option::Option<
        unsafe extern "C" fn(
            data: *mut ::std::os::raw::c_void,
            wl_registry: *mut wl_registry,
            name: u32,
        ),
    >,
}

pub fn init(wayland_client: &mut LibWaylandClient, wayland_display: *mut wl_display) -> bool {
    let registry = wayland_client.wl_display_get_registry(wayland_display);
    let registry_listener = wl_registry_listener {
        global: Some(registry_add_object),
        global_remove: Some(registry_remove_object),
    };
    unsafe {
        (wayland_client.wl_proxy_add_listener)(
            registry,
            &registry_listener as *const _ as _,
            wayland_client as *mut _ as _,
        );
        (wayland_client.wl_display_roundtrip)(wayland_display);

        if COMPOSITOR.is_null() {
            println!("wayland compositor failed");
            return false;
        }

        if SUBCOMPOSITOR.is_null() {
            println!("wayland subcompositor failed");
            return false;
        }

        if XDG_WM_BASE.is_null() {
            println!("wayland xdg_wm_base failed");
            return false;
        }

        if COMPOSITOR.is_null() {
            println!("wayland seat failed");
            return false;
        }

        true
    }
}
