use std::alloc::{Layout, alloc, dealloc, realloc};

#[unsafe(no_mangle)]
pub extern "C" fn __xs_malloc(size: usize, align: usize) -> *mut u8 {
    if let Ok(layout) = Layout::from_size_align(size, align) {
        unsafe {
            if layout.size() > 0 {
                let ptr = alloc(layout);
                if !ptr.is_null() {
                    return ptr;
                }
            } else {
                return align as *mut u8;
            }
        }
    }

    malloc_failure();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __xs_realloc(
    ptr: *mut u8,
    old_size: usize,
    new_size: usize,
    align: usize,
) -> *mut u8 {
    debug_assert!(old_size > 0);
    debug_assert!(new_size > 0);
    if let Ok(layout) = Layout::from_size_align(old_size, align) {
        unsafe {
            let ptr = realloc(ptr, layout, new_size);
            if !ptr.is_null() {
                return ptr;
            }
        }
    }
    malloc_failure();
}

#[cold]
fn malloc_failure() -> ! {
    core::arch::wasm32::unreachable();
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn __xs_free(ptr: *mut u8, size: usize, align: usize) {
    // This happens for zero-length slices, and in that case `ptr` is
    // likely bogus so don't actually send this to the system allocator
    if size == 0 {
        return;
    }
    unsafe {
        let layout = Layout::from_size_align_unchecked(size, align);
        dealloc(ptr, layout);
    }
}
