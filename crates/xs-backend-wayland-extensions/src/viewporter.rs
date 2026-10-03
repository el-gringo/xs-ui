#![allow(non_upper_case_globals, static_mut_refs)]

use crate::wayland_interface;

wayland_interface!(
    wp_viewport_interface,
    wp_viewport,
    3,
    [
        (destroy, "", ()),
        (set_source, "ffff", ()),
        (set_destination, "ii", ())
    ],
    []
);

wayland_interface!(
    wp_viewporter_interface,
    wp_viewporter,
    3,
    [
        (destroy, "", ()),
        (
            get_viewport,
            "no",
            (crate::viewporter::wp_viewport_interface)
        )
    ],
    []
);
