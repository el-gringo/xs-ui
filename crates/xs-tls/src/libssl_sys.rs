#![allow(non_camel_case_types, non_snake_case, dead_code)]

xs_platform_linux::declare_module! {
    LibSsl,
    "libssl.so",
    "libssl.so.3",
    "libssl.so.1",
    ...
    ...

    pub fn TLS_client_method() -> *const SSL_METHOD,

    pub fn OPENSSL_init_ssl(u64, *const ()) -> i32,

    pub fn OSSL_PROVIDER_load(*const (), *const c_char) -> *mut (),

    pub fn ERR_load_SSL_strings(),

    pub fn SSL_CTX_new(*const SSL_METHOD) -> *mut SSL_CTX,
    pub fn SSL_CTX_free(*mut SSL_CTX),

    pub fn BIO_new_ssl_connect(*mut SSL_CTX) -> *mut BIO,
    pub fn BIO_ctrl(*mut BIO, c_int, c_long, *mut c_void) -> c_long,
    pub fn BIO_read(*mut BIO, *mut u8, c_int) -> c_int,
    pub fn BIO_write(*mut BIO, *const u8, c_int) -> c_int,
    pub fn BIO_free_all(*mut BIO),

    ...
    ...
}

#[derive(Clone, Default)]
pub struct SSL_METHOD(pub *mut ());

#[derive(Clone, Default)]
pub struct SSL_CTX(pub *mut ());

#[derive(Clone, Default)]
pub struct SSL(pub *mut ());

#[derive(Clone, Default)]
pub struct BIO(pub *mut ());

#[derive(Clone, Default)]
pub struct SSL_CIPHER(pub *mut ());

pub type c_void = std::os::raw::c_void;
pub type c_int = std::os::raw::c_int;
pub type c_long = i64;
// `char` is unsigned on ARM, so this is `u8` there and `i8` on x86_64. Never
// hardcode `i8` in a declaration that a `CString::as_ptr()` is passed to: it
// only compiles on one of the two.
pub type c_char = std::os::raw::c_char;

pub const BIO_CTRL_FLUSH: c_int = 11;
pub const BIO_C_SET_CONNECT: c_int = 100;
pub const BIO_C_DO_STATE_MACHINE: c_int = 101;
pub const BIO_C_GET_SSL: c_int = 110;
