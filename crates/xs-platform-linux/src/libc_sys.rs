#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]

use std::mem;

#[cfg(target_pointer_width = "32")]
mod libc_32 {
    pub const ULONG_SIZE: usize = 32;
}
#[cfg(target_pointer_width = "32")]
pub use libc_32::*;

#[cfg(target_pointer_width = "64")]
mod libc_64 {
    pub const ULONG_SIZE: usize = 64;
}
#[cfg(target_pointer_width = "64")]
pub use libc_64::*;

pub type time_t = c_ulong;
pub type suseconds_t = c_ulong;

pub type c_int = std::os::raw::c_int;
pub type socklen_t = u32;
//type c_uint =  std::os::raw::c_uint;
pub type c_ulong = std::os::raw::c_ulong;
pub type c_void = std::os::raw::c_void;
pub type c_char = std::os::raw::c_char;
/// A count of bytes that can be -1 to report an error, which is what `read`
/// and friends return.
pub type ssize_t = isize;
pub type off_t = i64;
pub type size_t = usize;

pub const FD_SETSIZE: usize = 1024;
pub const EPIPE: c_int = 32;
pub const O_RDWR: c_int = 2;

pub const PROT_READ: c_int = 1;
pub const PROT_WRITE: c_int = 2;

pub const MAP_SHARED: c_int = 1;
pub const MAP_PRIVATE: c_int = 2;
pub const MAP_FAILED: *mut c_void = !0 as *mut c_void;

pub const O_CLOEXEC: c_int = 0x80000;

pub const IPPROTO_TCP: c_int = 6;
pub const TCP_NODELAY: c_int = 1;

#[repr(C)]
pub struct fd_set {
    fds_bits: [c_ulong; FD_SETSIZE / ULONG_SIZE],
}

pub const RTLD_LAZY: c_int = 1;
pub const RTLD_LOCAL: c_int = 0;
pub const RTLD_GLOBAL: c_int = 8;
pub const RTLD_NOW: c_int = 2;

unsafe extern "C" {
    pub fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    pub fn dlclose(handle: *mut c_void) -> c_int;
    pub fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    pub fn open(path: *const c_char, oflag: c_int, ...) -> c_int;
    pub fn close(fd: c_int) -> c_int;
    pub fn free(arg1: *mut c_void);
    pub fn pipe(fds: *mut c_int) -> c_int;
    pub fn select(
        nfds: c_int,
        readfds: *mut fd_set,
        writefds: *mut fd_set,
        errorfds: *mut fd_set,
        timeout: *mut timeval,
    ) -> c_int;
    pub fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t;
    pub fn unlink(c: *const c_char) -> c_int;
    pub fn mmap(
        addr: *mut c_void,
        len: size_t,
        prot: c_int,
        flags: c_int,
        fd: c_int,
        offset: off_t,
    ) -> *mut c_void;
    pub fn munmap(addr: *mut c_void, len: size_t) -> c_int;
    pub fn mkostemp(template: *mut c_char, flags: c_int) -> c_int;
    pub fn setsockopt(
        fd: c_int,
        level: c_int,
        optname: c_int,
        optval: *const c_void,
        optlen: socklen_t,
    ) -> c_int;
}

/// Adds `fd` to the set.
///
/// # Safety
///
/// `set` must point to a writable [`fd_set`], and `fd` must be below
/// `FD_SETSIZE`.
pub unsafe fn FD_SET(fd: c_int, set: *mut fd_set) {
    unsafe {
        let fd = fd as usize;
        let size = mem::size_of_val(&(*set).fds_bits[0]) * 8;
        (*set).fds_bits[fd / size] |= 1 << (fd % size);
    }
}

/// Clears every descriptor out of the set.
///
/// # Safety
///
/// `set` must point to a writable [`fd_set`].
pub unsafe fn FD_ZERO(set: *mut fd_set) {
    unsafe {
        for slot in (*set).fds_bits.iter_mut() {
            *slot = 0;
        }
    }
}

#[derive(Default, Clone, Copy)]
#[repr(C)]
pub struct timeval {
    pub tv_sec: time_t,
    pub tv_usec: suseconds_t,
}

pub enum FILE {}
