#![allow(static_mut_refs)]
use xs_backend_wayland_client::{wl_proxy, wl_request_constructor, wl_seat, wl_seat_capability, wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD, wl_seat_capability_WL_SEAT_CAPABILITY_POINTER, wl_seat_listener, LibWaylandClient, WL_SEAT_GET_KEYBOARD, WL_SEAT_GET_POINTER};

use crate::event::POINTER_LISTENER;


pub static mut SEAT_LISTENER: wl_seat_listener = wl_seat_listener {
    capabilities: Some(seat_handle_capabilities),
    name: Some(seat_handle_name),
};

unsafe extern "C" fn seat_handle_capabilities(
    data: *mut std::ffi::c_void,
    seat: *mut wl_seat,
    caps: wl_seat_capability,
) {
    unsafe {
        let client: &mut LibWaylandClient = &mut *(data as *mut _);
        if caps & wl_seat_capability_WL_SEAT_CAPABILITY_POINTER != 0 {
            // struct wl_pointer *pointer = wl_seat_get_pointer (seat);
            let id: *mut wl_proxy = wl_request_constructor!(
                client,
                seat,
                WL_SEAT_GET_POINTER,
                client.wl_pointer_interface
            );
            assert!(!id.is_null());
            // wl_pointer_add_listener (pointer, &pointer_listener, NULL);
            (client.wl_proxy_add_listener)(id, &POINTER_LISTENER as *const _ as _, data);
        }

        if caps & wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD != 0 {
            // struct wl_keyboard *keyboard = wl_seat_get_keyboard(seat);
            let id: *mut wl_proxy = wl_request_constructor!(
                client,
                seat,
                WL_SEAT_GET_KEYBOARD,
                client.wl_keyboard_interface
            );
            assert!(!id.is_null());
            // wl_keyboard_add_listener(keyboard, &keyboard_listener, NULL);
            //(client.wl_proxy_add_listener)(id, &KEYBOARD_LISTENER as *const _ as _, data);
        }
    }
}

extern "C" fn seat_handle_name(
    _data: *mut std::ffi::c_void,
    _seat: *mut wl_seat,
    _name: *const ::std::os::raw::c_char,
) {
}
