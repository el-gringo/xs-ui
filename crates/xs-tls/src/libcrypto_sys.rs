xs_platform_linux::declare_module! {
    LibCrypto,
    "libcrypto.so",
    "libcrypto.so.3",
    "libcrypto.so.1",
    ...
    ...

    pub fn OPENSSL_init_crypto(u64, *const ()) -> i32,

    pub fn OSSL_PROVIDER_load(*const (), *const i8) -> *mut (),

    pub fn EVP_CIPHER_CTX_new() -> *mut EVP_CIPHER_CTX,
    pub fn EVP_CIPHER_CTX_free(*mut EVP_CIPHER_CTX),
    pub fn EVP_CIPHER_CTX_reset(*mut EVP_CIPHER_CTX) -> i32,
    pub fn EVP_CIPHER_CTX_set_padding(*mut EVP_CIPHER_CTX, i32) -> i32,

    pub fn EVP_DecryptInit_ex(
        *mut EVP_CIPHER_CTX,
        *const EVP_CIPHER,
        *mut (),
        *const u8,
        *const u8,
    ) -> i32,

    pub fn EVP_DecryptUpdate(
        *mut EVP_CIPHER_CTX,
        *mut u8,
        *mut i32,
        *const u8,
        i32,
    ) -> i32,

    pub fn EVP_DecryptFinal_ex(
        *mut EVP_CIPHER_CTX,
        *mut u8,
        *mut i32,
    ) -> i32,

    pub fn EVP_EncryptInit_ex(
        *mut EVP_CIPHER_CTX,
        *const EVP_CIPHER,
        *mut (),
        *const u8,
        *const u8,
    ) -> i32,

    pub fn EVP_EncryptUpdate(
        *mut EVP_CIPHER_CTX,
        *mut u8,
        *mut i32,
        *const u8,
        i32,
    ) -> i32,

    pub fn EVP_EncryptFinal_ex(
        *mut EVP_CIPHER_CTX,
        *mut u8,
        *mut i32,
    ) -> i32,

    pub fn EVP_sha256() -> *const EVP_MD,
    pub fn EVP_sha1() -> *const EVP_MD,
    pub fn EVP_md5() -> *const EVP_MD,

    pub fn EVP_get_cipherbyname(*const i8) -> *const EVP_CIPHER,
    pub fn EVP_get_digestbyname(*const i8) -> *const EVP_MD,

    pub fn EVP_DigestInit_ex(
        *mut EVP_MD_CTX,
        *const EVP_MD,
        *mut (),
    ) -> i32,

    pub fn EVP_DigestUpdate(
        *mut EVP_MD_CTX,
        *const u8,
        usize,
    ) -> i32,

    pub fn EVP_DigestFinal_ex(
        *mut EVP_MD_CTX,
        *mut u8,
        *mut u32,
    ) -> i32,

    pub fn EVP_MD_CTX_new() -> *mut EVP_MD_CTX,
    pub fn EVP_MD_CTX_free(*mut EVP_MD_CTX),
    pub fn EVP_MD_CTX_reset(*mut EVP_MD_CTX) -> i32,

    pub fn HMAC_CTX_new() -> *mut HMAC_CTX,
    pub fn HMAC_CTX_free(*mut HMAC_CTX),
    pub fn HMAC_Init_ex(
        *mut HMAC_CTX,
        *const u8,
        i32,
        *const EVP_MD,
        *mut (),
    ) -> i32,
    pub fn HMAC_Update(*mut HMAC_CTX, *const u8, i32) -> i32,
    pub fn HMAC_Final(*mut HMAC_CTX, *mut u8, *mut u32) -> i32,

    pub fn RAND_bytes(*mut u8, i32) -> i32,

    pub fn X509_new() -> *mut X509,
    pub fn X509_free(*mut X509),
    pub fn X509_dup(*mut X509) -> *mut X509,
    pub fn X509_STORE_new() -> *mut X509_STORE,
    pub fn X509_STORE_free(*mut X509_STORE),
    pub fn X509_STORE_add_cert(*mut X509_STORE, *mut X509) -> i32,

    pub fn PEM_read_X509(
        *mut (),
        *mut *mut X509,
        *mut (),
        *mut (),
    ) -> *mut X509,

    pub fn PEM_read_PrivateKey(
        *mut (),
        *mut *mut (),
        *mut (),
        *mut (),
    ) -> *mut (),

    pub fn BIO_new(*const BIO_METHOD) -> *mut BIO,
    pub fn BIO_free(*mut BIO) -> i32,
    pub fn BIO_s_mem() -> *const BIO_METHOD,
    pub fn BIO_new_mem_buf(*const u8, i32) -> *mut BIO,

    pub fn BIO_read(*mut BIO, *mut u8, i32) -> i32,
    pub fn BIO_write(*mut BIO, *const u8, i32) -> i32,

    pub fn X509_get_subject_name(*mut X509) -> *mut X509_NAME,
    pub fn X509_NAME_get_index_by_NID(
        *mut X509_NAME,
        i32,
        i32,
    ) -> i32,
    pub fn X509_NAME_get_entry(*mut X509_NAME, i32) -> *mut X509_NAME_ENTRY,
    pub fn X509_NAME_ENTRY_get_data(*mut X509_NAME_ENTRY) -> *mut ASN1_STRING,
    pub fn ASN1_STRING_get0_data(*const ASN1_STRING) -> *const u8,
    pub fn ASN1_STRING_length(*const ASN1_STRING) -> i32,

    pub fn X509_verify_cert(*mut X509_STORE_CTX) -> i32,
    pub fn X509_STORE_CTX_new() -> *mut X509_STORE_CTX,
    pub fn X509_STORE_CTX_free(*mut X509_STORE_CTX),
    pub fn X509_STORE_CTX_init(
        *mut X509_STORE_CTX,
        *mut X509_STORE,
        *mut X509,
        *const (),
    ) -> i32,

    pub fn ERR_peek_error() -> u32,
    pub fn ERR_error_string_n(u32, *mut i8, usize),
    pub fn ERR_clear_error(),

    ...
    ...
}

#[derive(Clone, Default)]
pub struct EVP_CIPHER_CTX(pub *mut ());

#[derive(Clone, Default)]
pub struct EVP_MD_CTX(pub *mut ());

#[derive(Clone, Default)]
pub struct HMAC_CTX(pub *mut ());

#[derive(Clone, Default)]
pub struct X509(pub *mut ());

#[derive(Clone, Default)]
pub struct X509_STORE(pub *mut ());

#[derive(Clone, Default)]
pub struct X509_NAME(pub *mut ());

#[derive(Clone, Default)]
pub struct X509_NAME_ENTRY(pub *mut ());

#[derive(Clone, Default)]
pub struct ASN1_STRING(pub *mut ());

#[derive(Clone, Default)]
pub struct X509_STORE_CTX(pub *mut ());

#[derive(Clone, Default)]
pub struct BIO_METHOD(pub *mut ());

#[derive(Clone, Default)]
pub struct BIO(pub *mut ());

#[derive(Clone, Default)]
pub struct EVP_CIPHER(pub *mut ());

#[derive(Clone, Default)]
pub struct EVP_MD(pub *mut ());
