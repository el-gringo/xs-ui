use xs_platform_event::EventHandler;

use xs_backend_wayland_client::{
    wl_compositor,
    wl_keyboard,
    wl_pointer,
    wl_seat,
    wl_subcompositor,
    wl_surface,
    LibWaylandClient
};
use xs_backend_wayland_extensions::{xdg_shell, viewporter};
use xs_backend_wayland_egl::{wl_egl_window, LibWaylandEgl};
use xs_backend_wayland_xkb::xkb_keymap;

/// A thing to pass around within *void pointer of wayland's event handler
#[allow(dead_code)]
pub struct WaylandPayload {
    pub client: LibWaylandClient,
    // this is libwayland-egl.so, a library with ~4 functions
    // not the libEGL.so(which will be loaded, but not here)
    pub egl: LibWaylandEgl,

    //pub xkb: LibXkbCommon,
    //pub xkb_state: *mut xkb_state,
    //pub xkb_context: *mut xkb_context,
    pub compositor: *mut wl_compositor,
    pub subcompositor: *mut wl_subcompositor,
    pub xdg_toplevel: *mut xdg_shell::xdg_toplevel,
    pub xdg_wm_base: *mut xdg_shell::xdg_wm_base,
    pub surface: *mut wl_surface,

    pub viewporter: *mut viewporter::wp_viewporter,
    //pub shm: *mut wl_shm,
    pub seat: *mut wl_seat,
    pub keymap: *mut xkb_keymap,

    //pub wl_display:  *mut wl_display,
    pub egl_window: *mut wl_egl_window,
    pub pointer: *mut wl_pointer,
    pub keyboard: *mut wl_keyboard,
    pub focused_window: *mut wl_surface,

    pub event_handler: Option<Box<dyn EventHandler>>,
    pub closed: bool,
}

impl WaylandPayload {
    pub fn new(egl: LibWaylandEgl, client: LibWaylandClient) -> Self {
        Self  {
            egl,
            client,
            closed: false,
            compositor: std::ptr::null_mut(),
            subcompositor: std::ptr::null_mut(),
            xdg_toplevel: std::ptr::null_mut(),
            xdg_wm_base: std::ptr::null_mut(),
            surface: std::ptr::null_mut(),
            viewporter: std::ptr::null_mut(),
            //shm: std::ptr::null_mut(),
            seat: std::ptr::null_mut(),
            egl_window: std::ptr::null_mut(),
            event_handler: None,

            keymap: std::ptr::null_mut(),
            pointer: std::ptr::null_mut(),
            keyboard: std::ptr::null_mut(),
            focused_window: std::ptr::null_mut(),
        }
    }
}
