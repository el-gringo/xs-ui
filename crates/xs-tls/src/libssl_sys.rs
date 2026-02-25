#![allow(non_camel_case_types, non_snake_case, dead_code)]

xs_platform_linux::declare_module! {
    LibSsl,
    "libssl.so",
    "libssl.so.1",
    "libssl.so.1.1"
    ...
    ...

    pub fn SSL_library_init() -> i32,
    pub fn SSL_load_error_strings(),
    pub fn OpenSSL_add_all_algorithms(),

    pub fn TLS_method() -> *const SSL_METHOD,
    pub fn TLS_client_method() -> *const SSL_METHOD,
    pub fn TLS_server_method() -> *const SSL_METHOD,

    pub fn SSL_CTX_new(*const SSL_METHOD) -> *mut SSL_CTX,
    pub fn SSL_CTX_free(*mut SSL_CTX),
    pub fn SSL_CTX_up_ref(*mut SSL_CTX) -> i32,

    pub fn SSL_CTX_set_cipher_list(*mut SSL_CTX, *const i8) -> i32,
    pub fn SSL_CTX_set_alpn_protos(*mut SSL_CTX, *const u8, u32) -> i32,
    pub fn SSL_CTX_set_alpn_select_cb(
        *mut SSL_CTX,
        *const (),
        *mut u8,
    ),

    pub fn SSL_CTX_use_certificate_chain_file(
        *mut SSL_CTX,
        *const i8,
    ) -> i32,

    pub fn SSL_CTX_use_PrivateKey_file(
        *mut SSL_CTX,
        *const i8,
        i32,
    ) -> i32,

    pub fn SSL_CTX_check_private_key(*mut SSL_CTX) -> i32,

    pub fn SSL_new(*mut SSL_CTX) -> *mut SSL,
    pub fn SSL_free(*mut SSL),
    pub fn SSL_up_ref(*mut SSL) -> i32,

    pub fn SSL_set_fd(*mut SSL, i32) -> i32,
    pub fn SSL_get_fd(*mut SSL) -> i32,
    pub fn SSL_get_rfd(*mut SSL) -> i32,
    pub fn SSL_get_wfd(*mut SSL) -> i32,

    pub fn SSL_connect(*mut SSL) -> i32,
    pub fn SSL_accept(*mut SSL) -> i32,

    pub fn SSL_read(*mut SSL, *mut u8, i32) -> i32,
    pub fn SSL_read_ex(*mut SSL, *mut u8, usize, *mut usize) -> i32,
    pub fn SSL_write(*mut SSL, *const u8, i32) -> i32,
    pub fn SSL_write_ex(*mut SSL, *const u8, usize, *mut usize) -> i32,

    pub fn SSL_pending(*mut SSL) -> i32,
    pub fn SSL_has_pending(*mut SSL) -> i32,

    pub fn SSL_get_error(*mut SSL, i32) -> i32,
    pub fn SSL_get_shutdown(*mut SSL) -> i32,
    pub fn SSL_set_shutdown(*mut SSL, i32),

    pub fn SSL_get_version(*mut SSL) -> *const i8,
    pub fn SSL_get_cipher(*mut SSL) -> *const SSL_CIPHER,
    pub fn SSL_get_ciphers(*mut SSL) -> *mut stack_st_SSL_CIPHER,

    pub fn SSL_CTX_set_default_verify_paths(*mut SSL_CTX) -> i32,
    pub fn SSL_CTX_set_verify(
        *mut SSL_CTX,
        i32,
        *const (),
    ),
    pub fn SSL_CTX_set_verify_depth(*mut SSL_CTX, i32),

    pub fn SSL_get_verify_result(*mut SSL) -> i32,
    pub fn SSL_get_peer_certificate(*mut SSL) -> *mut crate::libcrypto_sys::X509,

    pub fn SSL_CTX_set_ecdh_auto(*mut SSL_CTX, i32),
    pub fn SSL_CTX_set1_curves_list(
        *mut SSL_CTX,
        *const i8,
    ) -> i32,

    pub fn SSL_set_connect_state(*mut SSL),
    pub fn SSL_set_accept_state(*mut SSL),

    pub fn SSL_set_tlsext_host_name(*mut SSL, *const i8) -> i32,

    pub fn SSL_get_current_cipher(*mut SSL) -> *mut SSL_CIPHER,
    pub fn SSL_CIPHER_get_name(*mut SSL_CIPHER) -> *const i8,
    pub fn SSL_CIPHER_get_version(*mut SSL_CIPHER) -> *const i8,
    pub fn SSL_CIPHER_get_bits(*mut SSL_CIPHER, *mut i32) -> i32,

    pub fn SSL_CTX_set_next_proto_select_cb(
        *mut SSL_CTX,
        *const (),
        *mut u8,
    ),

    pub fn SSL_get0_alpn_selected(*mut SSL, *mut *const u8, *mut u32),

    pub fn SSL_CTX_set_max_proto_version(*mut SSL_CTX, i32),
    pub fn SSL_CTX_set_min_proto_version(*mut SSL_CTX, i32),

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
pub struct SSL_CIPHER(pub *mut ());

#[derive(Clone, Default)]
pub struct stack_st_SSL_CIPHER(pub *mut ());
