use std::ffi::c_void;

use xs_backend_wayland_client::{wl_array, wl_keyboard, wl_keyboard_key_state, wl_keyboard_listener, wl_surface};

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
    _data: *mut c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: u32,
    _time: u32,
    _key: u32,
    _state: wl_keyboard_key_state,
) {
    // println!("keyboard_handle_key: {} {}", key, state);
}
