
xs_platform_linux::declare_module! {
    LibTls,
    "libssl.so",
    "libssl.so.1",
    ...
    ...

    SSL_CTX_new() -> SSL_CTX,
    TLS_client_method() -> SSL_METHOD,
    TLS_server_method() -> SSL_METHOD,

    SSL_CTX_use_certificate_chain_file(SSL_CTX ctx, )
    ...
    ...
}
