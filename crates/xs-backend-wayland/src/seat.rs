use xs_backend_wayland_client::{
    WL_SEAT_GET_KEYBOARD, wl_keyboard, wl_proxy,
    wl_request_constructor, wl_seat, wl_seat_capability,
    wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD, wl_seat_listener,
};

use crate::{context::WaylandContext, keyboard::{KEYBOARD_LISTENER, keyboard_handle_enter, keyboard_handle_key}};

pub unsafe extern "C" fn seat_handle_capabilities(
    data: *mut std::ffi::c_void,
    seat: *mut wl_seat,
    caps: wl_seat_capability,
) {
    assert!(!data.is_null());
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        if caps & wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD != 0 {
            let keyboard: *mut wl_keyboard = wl_request_constructor!(
                ctx.client,
                seat,
                WL_SEAT_GET_KEYBOARD,
                ctx.client.wl_keyboard_interface
            );
            assert!(!keyboard.is_null());
            KEYBOARD_LISTENER.enter = keyboard_handle_enter;
            KEYBOARD_LISTENER.key = keyboard_handle_key;
            (ctx.client.wl_proxy_add_listener)(
                keyboard as _,
                &raw const KEYBOARD_LISTENER as *const _ as _,
                data,
            );
        }
    }
}

pub static mut SEAT_LISTENER: wl_seat_listener = wl_seat_listener::dummy();

pub fn create_seat(ctx: &mut WaylandContext) {
    unsafe {
        SEAT_LISTENER.capabilities = seat_handle_capabilities;
        (ctx.client.wl_proxy_add_listener)(
            ctx.wl_seat as _,
            &raw const SEAT_LISTENER as *const _ as _,
            ctx as *mut _ as _
        );
    }
}
