use std::ffi::c_void;

use xs_backend_wayland_client::{
    wl_array, wl_keyboard, wl_keyboard_key_state, wl_keyboard_listener, wl_surface,
};

use crate::context::WaylandContext;

pub static mut KEYBOARD_LISTENER: wl_keyboard_listener = wl_keyboard_listener::dummy();

pub unsafe extern "C" fn keyboard_handle_enter(
    data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: ::core::ffi::c_uint,
    _surface: *mut wl_surface,
    _keys: *mut wl_array,
) {
    assert!(!data.is_null());
}

pub unsafe extern "C" fn keyboard_handle_key(
    data: *mut c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: u32,
    _time: u32,
    key: u32,
    state: wl_keyboard_key_state,
) {
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        if state == 1 {
            ctx.app_context.key_down = Some(key);
        }
    }
}
