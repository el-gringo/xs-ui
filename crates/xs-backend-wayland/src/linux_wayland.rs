#![allow(dead_code, static_mut_refs)]

use std::ffi::c_void;

use xs_backend_wayland_client::{
    wl_display, wl_output, wl_proxy, wl_request, wl_request_constructor, wl_surface, LibWaylandClient, WL_COMPOSITOR_CREATE_SURFACE, WL_SURFACE_COMMIT
};
use xs_backend_wayland_extensions::xdg_shell;
use xs_egl_sys::{LibEgl, create_egl_context};
use xs_layout::Layout;
use xs_renderer_core::Renderer;

use xs_backend_wayland_egl::{LibWaylandEgl, wl_egl_window};

use crate::config::ConfigWayland;
use crate::registry::{self, COMPOSITOR};
use crate::xdg;


fn try_load_libs() -> Option<(LibEgl, LibWaylandEgl, LibWaylandClient)> {
    if let Ok(lib_egl) = LibEgl::try_load() {
        if let Some(egl) = LibWaylandEgl::try_load() {
            if let Some(wayland_client) = LibWaylandClient::try_load() {
                return Some((lib_egl, egl, wayland_client));
            }
        }
    }
    None
}

pub fn window<F>(
    config: &ConfigWayland,
    renderer: &mut impl Renderer,
    generate_scene: F,
) -> Option<()>
where
    F: Fn() -> Box<dyn Layout>,
{
    unsafe {
        if let Some((mut lib_egl, egl, mut wayland_client)) = try_load_libs() {
            let wl_display = (wayland_client.wl_display_connect)(std::ptr::null_mut());
            if wl_display.is_null() {
                eprintln!("Failed to connect to Wayland display.");
                return None;
            }

            if !registry::init(&mut wayland_client, wl_display) {
                println!("creating wayland registry failed");
                return None;
            }

            let surface: *mut wl_surface = wl_request_constructor!(
                wayland_client,
                COMPOSITOR,
                WL_COMPOSITOR_CREATE_SURFACE,
                wayland_client.wl_surface_interface
            );
            if surface.is_null() {
                println!("creating wayland surface failed");
                return None;
            }

            let xdg_toplevel = xdg::init(&mut wayland_client, surface);
            if xdg_toplevel.is_null() {
                println!("xdg_toplevel failed");
                return None;
            }
            if let Ok(title) = std::ffi::CString::new(config.window_title.as_str()) {
                wl_request!(
                    wayland_client,
                    xdg_toplevel,
                    xdg_shell::xdg_toplevel::set_title,
                    title.as_ptr()
                );
            }

            wl_request!(wayland_client, surface, WL_SURFACE_COMMIT);

            let egl_window = (egl.wl_egl_window_create)(
                surface as _,
                config.window_width as _,
                config.window_height as _,
            );

            // For some reason, setting fullscreen before egl_window is created leads
            // to segfault because wl_egl_window_create returns NULL.
            if config.fullscreen {
                wl_request!(
                    wayland_client,
                    xdg_toplevel,
                    xdg_shell::xdg_toplevel::set_fullscreen,
                    std::ptr::null_mut::<*mut wl_output>()
                )
            }

            if let Some((egl_display, egl_surface)) = init_egl(&mut lib_egl, wl_display, egl_window)
            {
                renderer.init(lib_egl.eglGetProcAddress);
                //for _ in 0..2 {
                loop {
                    let widget_tree = generate_scene();
                    (wayland_client.wl_display_dispatch_pending)(wl_display);
                    renderer.draw_frame(widget_tree);
                    (lib_egl.eglSwapBuffers)(egl_display, egl_surface);
                }
                //}
            }
        }
    }
    None
}

pub fn init_egl(
    lib_egl: &mut LibEgl,
    wl_display: *mut wl_display,
    egl_window: *mut wl_egl_window,
) -> Option<(*mut c_void, *mut c_void)> {
    unsafe {
        if let Ok((egl_context, egl_config, egl_display)) =
            create_egl_context(lib_egl, wl_display as _, true)
        {
            let egl_surface = (lib_egl.eglCreateWindowSurface)(
                egl_display,
                egl_config,
                egl_window as _,
                std::ptr::null_mut(),
            );
            if egl_surface == /* EGL_NO_SURFACE  */ std::ptr::null_mut() {
                println!("eglCreateWindowSurface failed");
                return None;
            }

            if (lib_egl.eglMakeCurrent)(egl_display, egl_surface, egl_surface, egl_context) == 0 {
                println!("eglMakeCurrent failed");
                return None;
            }
            return Some((egl_display, egl_surface));
        }
    }
    None
}
