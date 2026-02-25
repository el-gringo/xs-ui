use std::ffi::c_void;

use xs_backend_wayland_client::{
    wl_array, wl_keyboard, wl_keyboard_key_state, wl_keyboard_listener, wl_surface,
};
use xs_platform_event::KeyEvent;
use xs_platform_linux::libc_sys;

use crate::{context::WaylandContext, keycodes::translate_keysym};

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
    data: *mut c_void,
    _wl_keyboard: *mut wl_keyboard,
    _serial: u32,
    _time: u32,
    key: u32,
    state: wl_keyboard_key_state,
) {
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        let lib_xkb = &mut ctx.lib_xkb;
        let mapping = ctx.xkb_keymap.mapping;
        let xkb_state = ctx.xkb_state;
        // https://wayland-book.com/seat/keyboard.html
        // To translate this to an XKB scancode, you must add 8 to the evdev scancode.
        let keysym = lib_xkb.keymap_key_get_sym_without_mod(mapping, key + 8);
        let code = translate_keysym(keysym);
        let mods = ctx.xkb_keymap.get_keymods(lib_xkb, xkb_state);
        if state == 1 {
            ctx.app.ctx.key_down = Some(KeyEvent::new(code, mods));
        }
    }
}

pub unsafe extern "C" fn keyboard_handle_keymap(
    data: *mut ::core::ffi::c_void,
    _wl_keyboard: *mut wl_keyboard,
    _format: u32,
    fd: i32,
    size: u32,
) {
    unsafe {
        let ctx: &mut WaylandContext = &mut *(data as *mut _);
        let map_shm = libc_sys::mmap(
            std::ptr::null_mut::<std::ffi::c_void>(),
            size as usize,
            libc_sys::PROT_READ,
            libc_sys::MAP_PRIVATE,
            fd,
            0,
        );
        assert!(map_shm != libc_sys::MAP_FAILED);
        (ctx.lib_xkb.xkb_keymap_unref)(ctx.xkb_keymap.mapping);
        ctx.xkb_keymap.mapping = (ctx.lib_xkb.xkb_keymap_new_from_string)(
            ctx.xkb_context,
            map_shm as *mut libc_sys::FILE,
            1,
            0,
        );
        libc_sys::munmap(map_shm, size as usize);
        libc_sys::close(fd);
        ctx.xkb_keymap.cache_mod_indices(&mut ctx.lib_xkb);
        (ctx.lib_xkb.xkb_state_unref)(ctx.xkb_state);
        ctx.xkb_state = (ctx.lib_xkb.xkb_state_new)(ctx.xkb_keymap.mapping);
    }
}
