use std::ffi::CString;
use std::io::{Read, Write};

use crate::libcrypto_sys::LibCrypto;
use crate::libssl_sys::{BIO_C_DO_STATE_MACHINE, BIO_C_SET_CONNECT, LibSsl};

pub struct TlsConnector {
    libssl: LibSsl,
    bio: *mut crate::libssl_sys::BIO,
}

unsafe impl Send for TlsConnector {}
unsafe impl Sync for TlsConnector {}

impl TlsConnector {
    pub fn connect(host: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let libcrypto = LibCrypto::try_load().expect("Unable to load libcrypto.so");
        let libssl = LibSsl::try_load().expect("Unable to load libssl.so");

        unsafe {
            let default_provider = CString::new("default")?;
            let _ = (libcrypto.OSSL_PROVIDER_load)(std::ptr::null(), default_provider.as_ptr());
            let legacy_provider = CString::new("legacy")?;
            let _ = (libcrypto.OSSL_PROVIDER_load)(std::ptr::null(), legacy_provider.as_ptr());

            let _ = (libssl.OPENSSL_init_ssl)(0x00100000 | 0x00200000, std::ptr::null());
            (libssl.ERR_load_SSL_strings)();
        }

        let method = unsafe { (libssl.TLS_client_method)() };
        let ctx = unsafe { (libssl.SSL_CTX_new)(method) };
        if ctx.is_null() {
            return Err("Failed to create SSL context".into());
        }

        let bio = unsafe { (libssl.BIO_new_ssl_connect)(ctx) };
        if bio.is_null() {
            unsafe { (libssl.SSL_CTX_free)(ctx) };
            return Err("Failed to create BIO".into());
        }

        let hostname = CString::new(format!("{}:{}", host, port))?;
        let ret = unsafe {
            (libssl.BIO_ctrl)(
                bio,
                BIO_C_SET_CONNECT,
                0,
                hostname.as_ptr() as *mut std::os::raw::c_void,
            )
        };
        if ret != 1 {
            unsafe { (libssl.BIO_free_all)(bio) };
            unsafe { (libssl.SSL_CTX_free)(ctx) };
            return Err("Failed to set hostname".into());
        }

        let ret = unsafe {
            (libssl.BIO_ctrl)(
                bio,
                BIO_C_DO_STATE_MACHINE,
                1,
                std::ptr::null_mut::<std::os::raw::c_void>(),
            )
        };
        if ret <= 0 {
            unsafe { (libssl.BIO_free_all)(bio) };
            unsafe { (libssl.SSL_CTX_free)(ctx) };
            return Err("Connection failed".into());
        }

        Ok(TlsConnector { libssl, bio })
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let ret = unsafe { (self.libssl.BIO_read)(self.bio, buf.as_mut_ptr(), buf.len() as i32) };
        if ret < 0 {
            return Err(std::io::Error::other("BIO read error"));
        }
        Ok(ret as usize)
    }

    pub fn write(&self, buf: &[u8]) -> Result<usize, std::io::Error> {
        let ret = unsafe { (self.libssl.BIO_write)(self.bio, buf.as_ptr(), buf.len() as i32) };
        if ret < 0 {
            return Err(std::io::Error::other("BIO write error"));
        }
        Ok(ret as usize)
    }

    pub fn flush(&self) -> Result<(), std::io::Error> {
        Ok(())
    }
}

impl Drop for TlsConnector {
    fn drop(&mut self) {
        unsafe {
            if !self.bio.is_null() {
                (self.libssl.BIO_free_all)(self.bio);
            }
        }
    }
}

pub struct TlsStream {
    tls: TlsConnector,
}

impl TlsStream {
    pub fn new(host: &str, port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        let tls = TlsConnector::connect(host, port)?;
        Ok(TlsStream { tls })
    }
}

impl Read for TlsStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        self.tls.read(buf)
    }
}

impl Write for TlsStream {
    fn write(&mut self, buf: &[u8]) -> Result<usize, std::io::Error> {
        self.tls.write(buf)
    }

    fn flush(&mut self) -> Result<(), std::io::Error> {
        self.tls.flush()
    }
}
