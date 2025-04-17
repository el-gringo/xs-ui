#![allow(static_mut_refs)]
use xs_backend_wayland_client::{wl_pointer, wl_pointer_listener, wl_surface};
use xs_platform_event::MouseButton;

pub enum WaylandEvent {
    KeyboardKey(u32, bool),
    PointerMotion(f32, f32),
    PointerButton(MouseButton, bool),
    PointerAxis(f32, f32),
}

pub static mut EVENTS: Vec<WaylandEvent> = Vec::new();
pub static mut POINTER_LISTENER: wl_pointer_listener = wl_pointer_listener {
    enter: Some(pointer_handle_enter),
    leave: Some(pointer_handle_leave),
    motion: Some(pointer_handle_motion),
    button: Some(pointer_handle_button),
    axis: Some(pointer_handle_axis),
    frame: Some(pointer_handle_frame),
    axis_source: Some(pointer_handle_axis_source),
    axis_stop: Some(pointer_handle_axis_stop),
    axis_discrete: Some(pointer_handle_axis_discrete),
    axis_value120: Some(pointer_handle_axis_value120),
    axis_relative_direction: Some(pointer_handle_axis_relative_direction),
};

fn wl_fixed_to_double(f: i32) -> f32 {
    (f as f32) / 256.0
}

unsafe extern "C" fn pointer_handle_enter(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _serial: u32,
    _surface: *mut wl_surface,
    _surface_x: i32,
    _surface_y: i32,
) {
}
unsafe extern "C" fn pointer_handle_leave(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _serial: u32,
    _surface: *mut wl_surface,
) {
}
unsafe extern "C" fn pointer_handle_motion(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _time: u32,
    surface_x: i32,
    surface_y: i32,
) {
    // From wl_fixed_to_double(), it simply divides by 256
    let (x, y) = (wl_fixed_to_double(surface_x), wl_fixed_to_double(surface_y));
    unsafe {
        EVENTS.push(WaylandEvent::PointerMotion(x, y));
    }
}
unsafe extern "C" fn pointer_handle_button(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _serial: u32,
    _time: u32,
    button: u32,
    state: u32,
) {
    // The code is defined in the kernel's linux/input-event-codes.h header file, e.g. BTN_LEFT
    let button = match button {
        272 => MouseButton::Left,
        273 => MouseButton::Right,
        274 => MouseButton::Middle,
        _ => MouseButton::Unknown,
    };
    unsafe {
        EVENTS.push(WaylandEvent::PointerButton(button, state == 1));
    }
}
unsafe extern "C" fn pointer_handle_axis(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _time: u32,
    axis: u32,
    value: i32,
) {
    let mut value = wl_fixed_to_double(value);
    // Normalize the value to {-1, 0, 1}
    value /= value.abs();

    // https://wayland-book.com/seat/pointer.html
    unsafe {
        if axis == 0 {
            // Vertical scroll
            // Wayland defines the direction differently to miniquad so lets flip it
            value = -value;
            EVENTS.push(WaylandEvent::PointerAxis(0.0, value));
        } else if axis == 1 {
            // Horizontal scroll
            EVENTS.push(WaylandEvent::PointerAxis(value, 0.0));
        }
    }
}
unsafe extern "C" fn pointer_handle_frame(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
) {
}
unsafe extern "C" fn pointer_handle_axis_source(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _axis_source: u32,
) {
}
unsafe extern "C" fn pointer_handle_axis_stop(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _time: u32,
    _axis: u32,
) {
}
unsafe extern "C" fn pointer_handle_axis_discrete(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _axis: u32,
    _discrete: i32,
) {
}
unsafe extern "C" fn pointer_handle_axis_value120(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _axis: u32,
    _value120: i32,
) {
}
unsafe extern "C" fn pointer_handle_axis_relative_direction(
    _data: *mut ::std::os::raw::c_void,
    _wl_pointer: *mut wl_pointer,
    _axis: u32,
    _direction: u32,
) {
}
