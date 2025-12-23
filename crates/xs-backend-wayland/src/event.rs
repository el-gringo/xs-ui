use xs_backend_wayland_client::{
    wl_array, wl_keyboard, wl_keyboard_key_state, wl_keyboard_listener, wl_pointer,
    wl_pointer_listener, wl_surface,
};
use xs_platform_event::MouseButton;

use crate::context::WaylandContext;

#[allow(dead_code)]
enum WaylandEvent {
    KeyboardKey(u32, bool),
    PointerMotion(f32, f32),
    PointerButton(MouseButton, bool),
    PointerAxis(f32, f32),
}

//pub static mut EVENTS: Vec<WaylandEvent> = Vec::new();
pub static mut KEYBOARD_LISTENER: wl_keyboard_listener = wl_keyboard_listener::dummy();
pub static mut POINTER_LISTENER: wl_pointer_listener = wl_pointer_listener::dummy();
//pub static mut TOUCH_LISTENER: wl_touch_listener = wl_touch_listener::dummy();


// fn wl_fixed_to_double(f: i32) -> f32 {
//     (f as f32) / 256.0
// }

//
// Pointer events
//

pub unsafe extern "C" fn pointer_handle_motion(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    time: u32,
    surface_x: i32,
    surface_y: i32,
) {
    // From wl_fixed_to_double(), it simply divides by 256
    // let (x, y) = (wl_fixed_to_double(surface_x), wl_fixed_to_double(surface_y));
    //EVENTS.push(WaylandEvent::PointerMotion(x, y));
}

// pub unsafe extern "C" fn pointer_handle_enter(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _serial: u32,
//     _surface: *mut wl_surface,
//     _surface_x: i32,
//     _surface_y: i32,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_leave(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _serial: u32,
//     _surface: *mut wl_surface,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_button(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _serial: u32,
//     _time: u32,
//     button: u32,
//     state: u32,
// ) {
//     // The code is defined in the kernel's linux/input-event-codes.h header file, e.g. BTN_LEFT
//     let button = match button {
//         272 => MouseButton::Left,
//         273 => MouseButton::Right,
//         274 => MouseButton::Middle,
//         _ => MouseButton::Unknown,
//     };
//     //EVENTS.push(WaylandEvent::PointerButton(button, state == 1));
// }

// pub unsafe extern "C" fn pointer_handle_axis(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _time: u32,
//     axis: u32,
//     value: i32,
// ) {
//     let mut value = wl_fixed_to_double(value);
//     // Normalize the value to {-1, 0, 1}
//     value /= value.abs();

//     // https://wayland-book.com/seat/pointer.html
//     if axis == 0 {
//         // Vertical scroll
//         // Wayland defines the direction differently to miniquad so lets flip it
//         value = -value;
//         //EVENTS.push(WaylandEvent::PointerAxis(0.0, value));
//     } else if axis == 1 {
//         // Horizontal scroll
//         //EVENTS.push(WaylandEvent::PointerAxis(value, 0.0));
//     }
// }

// pub unsafe extern "C" fn pointer_handle_frame(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_axis_source(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _axis_source: u32,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_axis_stop(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _time: u32,
//     _axis: u32,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_axis_discrete(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _axis: u32,
//     _discrete: i32,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_axis_value120(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _axis: u32,
//     _value120: i32,
// ) {
// }

// pub unsafe extern "C" fn pointer_handle_axis_relative_direction(
//     _data: *mut ::std::os::raw::c_void,
//     _wl_pointer: *mut wl_pointer,
//     _axis: u32,
//     _direction: u32,
// ) {
// }

//
// Keyboard events
//
pub unsafe extern "C" fn keyboard_handle_keymap(
    _data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _format: u32,
    _fd: i32,
    _size: u32,
) {
}

pub unsafe extern "C" fn keyboard_handle_enter(
    data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    serial: ::core::ffi::c_uint,
    _surface: *mut wl_surface,
    _keys: *mut wl_array,
) {
    assert!(!data.is_null());
    let ctx: &mut WaylandContext = &mut *(data as *mut _);
}

pub unsafe extern "C" fn keyboard_handle_leave(
    _data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: u32,
    _surface: *mut wl_surface,
) {
}

pub unsafe extern "C" fn keyboard_handle_key(
    _data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: u32,
    _time: u32,
    key: u32,
    state: wl_keyboard_key_state,
) {

}

pub unsafe extern "C" fn keyboard_handle_modifiers(
    _data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: u32,
    _mods_depressed: u32,
    _mods_latched: u32,
    _mods_locked: u32,
    _group: u32,
) {
}

pub unsafe extern "C" fn keyboard_handle_repeat_info(
    _data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _rate: i32,
    _delay: i32,
) {
}

//
// Touch events
//
// pub unsafe extern "C" fn touch_handle_down(
//     _data: *mut std::ffi::c_void,
//     _touch: *mut wl_touch,
//     _serial: core::ffi::c_uint,
//     _time: core::ffi::c_uint,
//     _surface: *mut wl_surface,
//     _id: core::ffi::c_int,
//     _x: wl_fixed_t,
//     _y: wl_fixed_t,
// ) {
// }

// pub unsafe extern "C" fn touch_handle_motion(
//     _data: *mut std::ffi::c_void,
//     _touch: *mut wl_touch,
//     _time: core::ffi::c_uint,
//     _id: core::ffi::c_int,
//     _x: wl_fixed_t,
//     _y: wl_fixed_t,
// ) {
// }

// pub unsafe extern "C" fn touch_handle_up(
//     _data: *mut std::ffi::c_void,
//     _touch: *mut wl_touch,
//     _serial: core::ffi::c_uint,
//     _time: core::ffi::c_uint,
//     _id: core::ffi::c_int,
// ) {
// }

// pub unsafe extern "C" fn touch_handle_cancel(_data: *mut std::ffi::c_void, _touch: *mut wl_touch) {}
>>>>>>> 7580656 (refactor(wayland): refactored wayland with initial implementation of seat)
