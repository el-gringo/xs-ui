#![allow(
    non_camel_case_types,
    non_upper_case_globals,
    non_snake_case,
    static_mut_refs,
    dead_code
)]

use crate::wayland_interface;
use xs_backend_wayland_client::{wl_array, wl_listener};

wayland_interface!(
    xdg_wm_base_interface,
    xdg_wm_base,
    3,
    [
        (destroy, "", ()),
        (
            create_positioner,
            "n",
            (crate::xdg_shell::xdg_positioner_interface)
        ),
        (
            get_xdg_surface,
            "no",
            (crate::xdg_shell::xdg_surface_interface)
        ),
        (pong, "u", ())
    ],
    [("ping", "u")]
);

wayland_interface!(
    xdg_surface_interface,
    xdg_surface,
    3,
    [
        (destroy, "", ()),
        (
            get_toplevel,
            "n",
            (crate::xdg_shell::xdg_toplevel_interface)
        ),
        (get_popup, "n?oo", ()),
        (set_window_geometry, "iiii", ()),
        (ack_configure, "u", ())
    ],
    [("configure", "u")]
);

wayland_interface!(
    xdg_toplevel_interface,
    xdg_toplevel,
    3,
    [
        (destroy, "", ()),
        (set_parent, "?o", (xdg_toplevel_interface)),
        (set_title, "s", ()),
        (set_app_id, "s", ()),
        (
            show_window_menu,
            "ouii",
            (crate::xdg_shell::xdg_popup_interface)
        ),
        (
            r#move,
            "ou",
            (crate::extensions::wayland_protocol::wl_seat_interface)
        ),
        (
            resize,
            "ouu",
            (crate::extensions::wayland_protocol::wl_seat_interface)
        ),
        (set_max_size, "ii", ()),
        (set_min_size, "ii", ()),
        (set_maximized, "", ()),
        (unset_maximized, "", ()),
        (
            set_fullscreen,
            "?o",
            (crate::extensions::wayland_protocol::wl_output_interface)
        ),
        (unset_fullscreen, "", ()),
        (set_minimized, "", ())
    ],
    [("configure", "iia"), ("close", "")]
);

wayland_interface!(
    xdg_positioner_interface,
    xdg_positioner,
    3,
    [
        (destroy, "", ()),
        (set_size, "ii", ()),
        (set_anchor_rect, "iiii", ()),
        (set_anchor, "u", ()),
        (set_gravity, "u", ()),
        (set_constraint_adjustment, "u", ()),
        (set_offset, "ii", ()),
        (set_reactive, "3", ()),
        (set_parent_size, "3ii", ()),
        (set_parent_configure, "3u", ())
    ],
    []
);

wayland_interface!(
    xdg_popup_interface,
    xdg_popup,
    3,
    [
        (destroy, "", ()),
        (
            grab,
            "ou",
            (crate::extensions::wayland_protocol::wl_seat_interface)
        ),
        (
            reposition,
            "3ou",
            (crate::xdg_shell::xdg_positioner_interface)
        )
    ],
    [("configure", "iiii"), ("popup_done", "")]
);

wl_listener!(
    xdg_wm_base_listener,
    xdg_wm_base,
    xdg_wm_base_dummy,
    fn ping(serial: core::ffi::c_uint),
);

wl_listener!(
    xdg_surface_listener,
    xdg_surface,
    xdg_surface_dummy,
    fn configure(serial: core::ffi::c_uint),
);

wl_listener!(
    xdg_toplevel_listener,
    xdg_toplevel,
    xdg_toplevel_dummy,
    fn configure(
        width: core::ffi::c_int,
        height: core::ffi::c_int,
        states: *mut wl_array,
    ),
    fn close(),
);
