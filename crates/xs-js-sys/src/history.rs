mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn history_push_state(url_ptr: *const u8, url_len: usize);
        pub fn history_replace_state(url_ptr: *const u8, url_len: usize);
        pub fn history_back();
        pub fn history_forward();
        pub fn history_go(delta: i32);
        pub fn location_path() -> u32;
    }
}

/// Pushes a new browser history entry.
///
/// The url is resolved by the browser itself, against the current location, so
/// a relative target behaves exactly as it does in the address bar.
pub fn history_push_state(url: &str) {
    unsafe {
        js::history_push_state(url.as_ptr(), url.len());
    }
}

/// Replaces the current browser history entry.
pub fn history_replace_state(url: &str) {
    unsafe {
        js::history_replace_state(url.as_ptr(), url.len());
    }
}

pub fn history_back() {
    unsafe {
        js::history_back();
    }
}

pub fn history_forward() {
    unsafe {
        js::history_forward();
    }
}

/// Moves `delta` entries. The browser clamps a delta outside the stack.
pub fn history_go(delta: i32) {
    unsafe {
        js::history_go(delta);
    }
}

/// The current `pathname + search + hash`.
///
/// Only the path is handed back, not the origin: a router lives in one origin,
/// and the origin is not part of a route.
#[must_use]
pub fn location_path() -> String {
    unsafe { read_c_string(js::location_path()) }
}

/// Reads a string the bindings wrote in wasm memory as a null terminated c
/// string. The buffer belongs to the heap allocator and is not freed.
unsafe fn read_c_string(ptr: u32) -> String {
    unsafe {
        std::ffi::CStr::from_ptr(ptr as *const std::ffi::c_char)
            .to_string_lossy()
            .into_owned()
    }
}
