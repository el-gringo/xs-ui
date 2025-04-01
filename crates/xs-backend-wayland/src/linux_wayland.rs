#![allow(dead_code, static_mut_refs)]

use std::ffi::c_void;

use xs_layout::Layout;
use xs_platform_event::MouseButton;
use xs_renderer_core::Renderer;
use xs_egl_sys::{LibEgl, create_egl_context};
use xs_backend_wayland_extensions::{xdg_shell, viewporter};
use xs_backend_wayland_client::{
    WL_COMPOSITOR_CREATE_SURFACE,
    WL_SEAT_GET_KEYBOARD,
    WL_SEAT_GET_POINTER,
    WL_SURFACE_COMMIT,

    wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD,
    wl_seat_capability_WL_SEAT_CAPABILITY_POINTER,

    wl_array,
    wl_display,
    wl_output,
    wl_pointer,
    wl_pointer_listener,
    wl_proxy,
    wl_registry,
    wl_registry_listener,
    wl_seat,
    wl_seat_capability,
    wl_seat_listener,
    wl_surface,
    wl_request,
    wl_request_constructor,
    LibWaylandClient
};

use xs_backend_wayland_egl::{wl_egl_window, LibWaylandEgl};

use crate::payload::WaylandPayload;
use crate::config::ConfigWayland;


fn wl_fixed_to_double(f: i32) -> f32 {
    (f as f32) / 256.0
}

static mut SEAT_LISTENER: wl_seat_listener = wl_seat_listener {
    capabilities: Some(seat_handle_capabilities),
    name: Some(seat_handle_name),
};

unsafe extern "C" fn seat_handle_capabilities(
    data: *mut std::ffi::c_void,
    seat: *mut wl_seat,
    caps: wl_seat_capability,
) {
    unsafe {
        let display: &mut WaylandPayload = &mut *(data as *mut _);
        if caps & wl_seat_capability_WL_SEAT_CAPABILITY_POINTER != 0 {
            // struct wl_pointer *pointer = wl_seat_get_pointer (seat);
            let id: *mut wl_proxy = wl_request_constructor!(
                display.client,
                seat,
                WL_SEAT_GET_POINTER,
                display.client.wl_pointer_interface
            );
            assert!(!id.is_null());
            // wl_pointer_add_listener (pointer, &pointer_listener, NULL);
            (display.client.wl_proxy_add_listener)(id, &POINTER_LISTENER as *const _ as _, data);
        }

        if caps & wl_seat_capability_WL_SEAT_CAPABILITY_KEYBOARD != 0 {
            // struct wl_keyboard *keyboard = wl_seat_get_keyboard(seat);
            let id: *mut wl_proxy = wl_request_constructor!(
                display.client,
                seat,
                WL_SEAT_GET_KEYBOARD,
                display.client.wl_keyboard_interface
            );
            assert!(!id.is_null());
            // wl_keyboard_add_listener(keyboard, &keyboard_listener, NULL);
            //(display.client.wl_proxy_add_listener)(id, &KEYBOARD_LISTENER as *const _ as _, data);
        }
    }
}

enum WaylandEvent {
    KeyboardKey(u32, bool),
    PointerMotion(f32, f32),
    PointerButton(MouseButton, bool),
    PointerAxis(f32, f32),
}

static mut EVENTS: Vec<WaylandEvent> = Vec::new();


static mut POINTER_LISTENER: wl_pointer_listener = wl_pointer_listener {
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

extern "C" fn seat_handle_name(
    _data: *mut std::ffi::c_void,
    _seat: *mut wl_seat,
    _name: *const ::std::os::raw::c_char,
) {
}

unsafe extern "C" fn registry_add_object(
    data: *mut std::ffi::c_void,
    registry: *mut wl_registry,
    name: u32,
    interface: *const ::std::os::raw::c_char,
    version: u32,
) {
    unsafe {
        let display: &mut WaylandPayload = &mut *(data as *mut _);
        let interface = std::ffi::CStr::from_ptr(interface).to_str().unwrap();
        match interface {
            "wl_compositor" => {
                display.compositor = display.client.wl_registry_bind(
                    registry,
                    name,
                    display.client.wl_compositor_interface,
                    1,
                ) as _;
            }
            "wl_subcompositor" => {
                display.subcompositor = display.client.wl_registry_bind(
                    registry,
                    name,
                    display.client.wl_subcompositor_interface,
                    1,
                ) as _;
            }
            "xdg_wm_base" => {
                display.xdg_wm_base = display.client.wl_registry_bind(
                    registry,
                    name,
                    &xdg_shell::xdg_wm_base_interface,
                    1,
                ) as _;
            }
            "wp_viewporter" => {
                display.viewporter = display.client.wl_registry_bind(
                    registry,
                    name,
                    &viewporter::wp_viewporter_interface,
                    1,
                ) as _;
            }
            "wl_seat" => {
                let seat_version = 4.min(version);
                display.seat = display.client.wl_registry_bind(
                    registry,
                    name,
                    display.client.wl_seat_interface,
                    seat_version,
                ) as _;
                (display.client.wl_proxy_add_listener)(
                    display.seat as _,
                    &SEAT_LISTENER as *const _ as _,
                    data,
                );
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

unsafe extern "C" fn xdg_surface_handle_configure(
    data: *mut std::ffi::c_void,
    xdg_surface: *mut xdg_shell::xdg_surface,
    serial: u32,
) {
    unsafe {
        assert!(!data.is_null());
        let payload: &mut WaylandPayload = &mut *(data as *mut _);
        wl_request!(
            payload.client,
            xdg_surface,
            xdg_shell::xdg_surface::ack_configure,
            serial
        );
        wl_request!(payload.client, payload.surface, WL_SURFACE_COMMIT)
    }
}

unsafe extern "C" fn xdg_toplevel_handle_close(
    data: *mut std::ffi::c_void,
    _xdg_toplevel: *mut xdg_shell::xdg_toplevel,
) {
    unsafe {
        assert!(!data.is_null());
        let payload: &mut WaylandPayload = &mut *(data as *mut _);
        payload.closed = true;
    }
}

unsafe extern "C" fn xdg_toplevel_handle_configure(
    data: *mut std::ffi::c_void,
    _toplevel: *mut xdg_shell::xdg_toplevel,
    width: i32,
    height: i32,
    _states: *mut wl_array,
) -> () {
    unsafe {
        assert!(!data.is_null());
        let payload: &mut WaylandPayload = &mut *(data as *mut _);
        if width != 0 && height != 0 {
            (payload.egl.wl_egl_window_resize)(payload.egl_window, width, height, 0, 0);
            if let Some(ref mut event_handler) = payload.event_handler {
                event_handler.resize_event(width as _, height as _);
            }
        }
    }
}

unsafe extern "C" fn xdg_wm_base_handle_ping(
    data: *mut std::ffi::c_void,
    toplevel: *mut xdg_shell::xdg_wm_base,
    serial: u32,
) -> () {
    unsafe {
        assert!(!data.is_null());
        let payload: &mut WaylandPayload = &mut *(data as *mut _);

        wl_request!(
            payload.client,
            toplevel,
            xdg_shell::xdg_wm_base::pong,
            serial
        );
    }
}

fn try_load_libs() -> Option<(LibEgl, LibWaylandEgl, LibWaylandClient)> {
    if let Ok(lib_egl) = LibEgl::try_load() {
        if let Some(egl) = LibWaylandEgl::try_load() {
            if let Some(wayland_client) = LibWaylandClient::try_load() {
                return Some((lib_egl, egl, wayland_client))
            }
        }
    }
    None
}


pub fn window<F>(conf: &ConfigWayland, renderer: &mut impl Renderer, generate_scene: F) -> Option<()>
where
    F: Fn() -> Box<dyn Layout> {
    unsafe {
        if let Some((mut lib_egl, egl, mut wayland_client)) = try_load_libs() {
            let wl_display = (wayland_client.wl_display_connect)(std::ptr::null_mut());
            if wl_display.is_null() {
                eprintln!("Failed to connect to Wayland display.");
                return None;
            }
            let registry = wayland_client.wl_display_get_registry(wl_display);
            let registry_listener = wl_registry_listener {
                global: Some(registry_add_object),
                global_remove: Some(registry_remove_object),
            };
            let mut display = WaylandPayload::new(egl, wayland_client);
            (display.client.wl_proxy_add_listener)(
                registry,
                &registry_listener as *const _ as _,
                &mut display as *mut _ as _,
            );
            (display.client.wl_display_roundtrip)(wl_display);

            assert!(display.compositor.is_null() == false);
            assert!(display.xdg_wm_base.is_null() == false);
            assert!(display.subcompositor.is_null() == false);
            assert!(display.seat.is_null() == false);
            //assert!(display.keymap.is_null() == false);
            //assert!(display.xkb_state.is_null() == false);

            let xdg_wm_base_listener = xdg_shell::xdg_wm_base_listener {
                ping: Some(xdg_wm_base_handle_ping),
            };

            (display.client.wl_proxy_add_listener)(
                display.xdg_wm_base as _,
                &xdg_wm_base_listener as *const _ as _,
                &mut display as *mut _ as _,
            );

            display.surface = wl_request_constructor!(
                display.client,
                display.compositor,
                WL_COMPOSITOR_CREATE_SURFACE,
                display.client.wl_surface_interface
            );
            assert!(display.surface.is_null() == false);

            let xdg_surface: *mut xdg_shell::xdg_surface = wl_request_constructor!(
                display.client,
                display.xdg_wm_base,
                xdg_shell::xdg_wm_base::get_xdg_surface,
                &xdg_shell::xdg_surface_interface,
                display.surface
            );
            assert!(xdg_surface.is_null() == false);

            let xdg_surface_listener = xdg_shell::xdg_surface_listener {
                configure: Some(xdg_surface_handle_configure),
            };

            (display.client.wl_proxy_add_listener)(
                xdg_surface as _,
                &xdg_surface_listener as *const _ as _,
                &mut display as *mut _ as _,
            );

            display.xdg_toplevel = wl_request_constructor!(
                display.client,
                xdg_surface,
                xdg_shell::xdg_surface::get_toplevel,
                &xdg_shell::xdg_toplevel_interface
            );
            assert!(display.xdg_toplevel.is_null() == false);

            let xdg_toplevel_listener = xdg_shell::xdg_toplevel_listener {
                configure: Some(xdg_toplevel_handle_configure),
                close: Some(xdg_toplevel_handle_close),
            };

            (display.client.wl_proxy_add_listener)(
                display.xdg_toplevel as _,
                &xdg_toplevel_listener as *const _ as _,
                &mut display as *mut _ as _,
            );

            if let Ok(title) = std::ffi::CString::new(conf.window_title.as_str()) {
                wl_request!(
                    display.client,
                    display.xdg_toplevel,
                    xdg_shell::xdg_toplevel::set_title,
                    title.as_ptr()
                );
            }

            wl_request!(display.client, display.surface, WL_SURFACE_COMMIT);

            let egl_window = (display.egl.wl_egl_window_create)(
                display.surface as _,
                conf.window_width as _,
                conf.window_height as _,
            );
            display.egl_window = egl_window;

            // For some reason, setting fullscreen before egl_window is created leads
            // to segfault because wl_egl_window_create returns NULL.
            if conf.fullscreen {
                wl_request!(
                    display.client,
                    display.xdg_toplevel,
                    xdg_shell::xdg_toplevel::set_fullscreen,
                    std::ptr::null_mut::<*mut wl_output>()
                )
            }

            if let Some((egl_display, egl_surface)) = init_egl(&mut lib_egl, wl_display, egl_window) {
                renderer.init(lib_egl.eglGetProcAddress);
                //for _ in 0..2 {
                loop {
                    let widget_tree = generate_scene();
                    (display.client.wl_display_dispatch_pending)(wl_display);
                    renderer.draw_frame(widget_tree);
                    (lib_egl.eglSwapBuffers)(egl_display, egl_surface);
                }
                    //}
            }
        }
    }
    None
}

pub fn init_egl(lib_egl: &mut LibEgl, wl_display: *mut wl_display, egl_window: *mut wl_egl_window) -> Option<(*mut c_void, *mut c_void)> {
    unsafe {
        if let Ok((
            egl_context,
            egl_config,
            egl_display
        )) = create_egl_context(
                lib_egl,
                wl_display as _,
                true
        ) {
            let egl_surface = (lib_egl.eglCreateWindowSurface)(
                egl_display,
                egl_config,
                egl_window as _,
                std::ptr::null_mut(),
            );
            if egl_surface == /* EGL_NO_SURFACE  */ std::ptr::null_mut() {
                panic!("eglCreateWindowSurface failed");
            }

            if (lib_egl.eglMakeCurrent)(egl_display, egl_surface, egl_surface, egl_context) == 0 {
                panic!("eglMakeCurrent failed");
            }

            return Some((egl_display, egl_surface));
        }
    }
    None
}
