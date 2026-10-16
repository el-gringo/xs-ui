use std::{ffi::c_void, ptr::null_mut};

use xs_backend_wayland_client::{
    LibWaylandClient, WL_SURFACE_COMMIT, WL_SURFACE_SET_BUFFER_SCALE, wl_compositor, wl_display,
    wl_output, wl_registry, wl_request, wl_seat, wl_surface,
};
use xs_backend_wayland_egl::{LibWaylandEgl, wl_egl_window};
use xs_backend_wayland_extensions::xdg_shell::{xdg_surface, xdg_toplevel, xdg_wm_base};
use xs_backend_wayland_xkb::{LibXkbCommon, XkbKeymap, xkb_context, xkb_state};
use xs_core::AppContext;
use xs_egl_sys::LibEgl;

pub struct WaylandContext<'a> {
    pub app_context: &'a mut AppContext,

    pub client: LibWaylandClient,
    pub lib_wayland_egl: LibWaylandEgl,
    pub lib_egl: LibEgl,
    pub lib_xkb: LibXkbCommon,

    pub egl_display: *mut c_void,
    pub egl_window: *mut wl_egl_window,
    pub egl_surface: *mut c_void,
    pub egl_config: *mut c_void,
    pub egl_context: *mut c_void,

    pub wl_display: *mut wl_display,
    pub wl_surface: *mut wl_surface,
    pub wl_registry: *mut wl_registry,
    pub wl_seat: *mut wl_seat,
    pub wl_compositor: *mut wl_compositor,
    pub wl_output: *mut wl_output,

    pub xdg_wm_base: *mut xdg_wm_base,
    pub xdg_toplevel: *mut xdg_toplevel,
    pub xdg_surface: *mut xdg_surface,

    pub xkb_context: *mut xkb_context,
    pub xkb_state: *mut xkb_state,
    pub xkb_keymap: XkbKeymap,

    /// Whether `wl_surface.preferred_buffer_scale` has reported yet.
    ///
    /// That event outranks `wl_output.scale`: the protocol says so, it is the
    /// scale for *this* surface rather than the output's, and it is the one a
    /// compositor is required to keep current. `wl_output.scale` is a fallback
    /// for the older protocol version, and the flag is what keeps the two from
    /// fighting - see [`WaylandContext::apply_scale`].
    pub surface_scale_known: bool,
}

impl<'a> WaylandContext<'a> {
    pub fn new(app_context: &'a mut AppContext) -> WaylandContext<'a> {
        let client = LibWaylandClient::try_load().expect("failed to load libwayland-client.so");
        let lib_wayland_egl = LibWaylandEgl::try_load().expect("failed to load libwayland-egl.so");
        let lib_egl = LibEgl::try_load().expect("failed to load libEGL.so");
        let lib_xkb = LibXkbCommon::try_load().expect("failed to load libxkbcommon.so");
        WaylandContext {
            app_context,

            client,
            lib_wayland_egl,
            lib_egl,
            lib_xkb,

            egl_display: null_mut(),
            egl_window: null_mut(),
            egl_surface: null_mut(),
            egl_config: null_mut(),
            egl_context: null_mut(),

            wl_display: null_mut(),
            wl_compositor: null_mut(),
            wl_registry: null_mut(),
            wl_surface: null_mut(),
            wl_seat: null_mut(),
            wl_output: null_mut(),

            xdg_wm_base: null_mut(),
            xdg_surface: null_mut(),
            xdg_toplevel: null_mut(),

            xkb_context: null_mut(),
            xkb_state: null_mut(),
            xkb_keymap: Default::default(),
            surface_scale_known: false,
        }
    }

    pub fn round_trip(&mut self) {
        unsafe {
            (self.client.wl_display_roundtrip)(self.wl_display);
        }
    }

    /// The surface size in **buffer** pixels, which is what
    /// `wl_egl_window_create` and `wl_egl_window_resize` are given.
    ///
    /// Wayland reports its sizes in surface-local pixels, so this is where the
    /// compositor's number gets turned into the number of pixels the GPU
    /// actually rasterises. It is the native counterpart of the browser's
    /// `canvas.width = cssWidth * devicePixelRatio`, and it reads the same
    /// `Screen::framebuffer_size` the GL viewport does, which is what keeps the
    /// two in step.
    pub fn buffer_size(&self) -> (i32, i32) {
        let (width, height) = self.app_context.screen.framebuffer_size();
        (width as i32, height as i32)
    }

    pub fn sufarce_commit(&mut self) {
        unsafe {
            wl_request!(self.client, self.wl_surface, WL_SURFACE_COMMIT);
        }
    }

    /// Tells the compositor how many buffer pixels there are per surface-local
    /// pixel.
    ///
    /// This is the Wayland half of the browser's `devicePixelRatio`, and it
    /// works the same way in reverse: with it set to `s` the surface keeps
    /// reporting `width`/`height` in surface-local pixels to the app and to
    /// `xdg_toplevel.configure`, while the buffers it expects are `s` times
    /// that. Without it a HiDPI compositor would read our `logical * s` buffer
    /// as a surface `s` times the intended size, and the window would be
    /// enormous.
    ///
    /// The scale is an integer here - `wl_surface.set_buffer_scale` takes one -
    /// which is all a compositor offers. A fractional ratio is what a browser
    /// reports on a scaled desktop, and there the browser does the scaling
    /// itself because the canvas is a bitmap it can resample.
    pub fn set_buffer_scale(&mut self, scale: f32) {
        let scale = scale.round().max(1.0) as i32;
        unsafe {
            wl_request!(
                self.client,
                self.wl_surface,
                WL_SURFACE_SET_BUFFER_SCALE,
                scale
            );
        }
    }

    /// Puts a scale reading on the screen, and rebuilds the drawing buffer for
    /// it if there is one yet.
    ///
    /// `from_surface_hint` says which of the two events it came from. Only the
    /// surface hint is allowed to override an existing reading, because
    /// `wl_output.scale` is optional - its own documentation says "if it is not
    /// sent, the client should assume a scale of 1", and the compositor this was
    /// developed against advertises `wl_output` version 4 while sending
    /// `geometry`, `mode` and `done` and no `scale` at all. A client reading
    /// only that one stays pinned at 1.0 on a HiDPI screen.
    pub fn apply_scale(&mut self, scale: f32, from_surface_hint: bool) {
        if from_surface_hint {
            self.surface_scale_known = true;
        } else if self.surface_scale_known {
            return;
        }
        if (self.app_context.screen.scale - scale).abs() < f32::EPSILON {
            return;
        }
        self.app_context.screen.set_scale(scale);
        // Before anything is created there is nothing to rebuild:
        // `BackendWayland::run` reads the scale off the screen when it announces
        // the buffer scale and sizes the egl window. After that, the buffer has
        // to be resized immediately or the surface is a stretched or letterboxed
        // image, and it has to be marked dirty because resizing clears it and a
        // frame is only drawn when something asks for one.
        if self.egl_window.is_null() {
            return;
        }
        self.set_buffer_scale(scale);
        let (width, height) = self.buffer_size();
        unsafe {
            (self.lib_wayland_egl.wl_egl_window_resize)(self.egl_window, width, height, 0, 0);
        }
        self.app_context.dirty = true;
    }

    pub fn dispatch_pending(&mut self) -> i32 {
        unsafe { (self.client.wl_display_dispatch_pending)(self.wl_display) }
    }

    pub fn dispatch(&mut self) -> i32 {
        unsafe { (self.client.wl_display_dispatch)(self.wl_display) }
    }

    pub fn swap_buffers(&mut self) {
        unsafe {
            (self.lib_egl.eglSwapBuffers)(self.egl_display, self.egl_surface);
        }
    }
}
