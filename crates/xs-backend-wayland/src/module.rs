#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
    DlOpenError(String),
    DlSymError(String),
}

use libc::{dlclose, dlopen, dlsym, RTLD_LAZY, RTLD_LOCAL};
use std::{
    ffi::{c_void, CString},
    ptr::NonNull,
};

pub struct Module(NonNull<c_void>);

impl Module {
    pub fn load(path: &str) -> Result<Self, Error> {
        let cpath = CString::new(path).unwrap();
        let module = unsafe { dlopen(cpath.as_ptr(), RTLD_LAZY | RTLD_LOCAL) };
        if module.is_null() {
            Err(Error::DlOpenError(path.to_string()))
        } else {
            Ok(Module(unsafe { NonNull::new_unchecked(module) }))
        }
    }

    pub fn get_symbol<F: Sized>(&self, name: &str) -> Result<F, Error> {
        let cname = CString::new(name).unwrap();
        let symbol = unsafe { dlsym(self.0.as_ptr(), cname.as_ptr()) };
        if symbol.is_null() {
            return Err(Error::DlSymError(name.to_string()));
        }
        Ok(unsafe { std::mem::transmute_copy::<_, F>(&symbol) })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        unsafe { dlclose(self.0.as_ptr()) };
    }
}
