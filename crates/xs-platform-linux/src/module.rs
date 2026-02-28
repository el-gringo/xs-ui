use crate::libc_sys::{RTLD_GLOBAL, RTLD_NOW, dlclose, dlopen, dlsym};
use std::{ffi::CString, ptr::NonNull};

#[derive(PartialEq, Eq, Clone)]
pub enum Error {
    DlOpenError(String),
    DlSymError(String),
}

/// A module loader for loading shared libraries.
pub struct Module(::std::ptr::NonNull<::std::os::raw::c_void>);

impl Module {
    /// Load a shared library by its path.
    pub fn load(path: &str) -> Result<Self, ()> {
        Self::load_with_flags(path, RTLD_NOW | RTLD_GLOBAL)
    }

    /// Load a shared library by its path with RTLD_GLOBAL to make symbols available
    /// to subsequently loaded libraries.
    pub fn load_global(path: &str) -> Result<Self, ()> {
        Self::load_with_flags(path, RTLD_NOW | RTLD_GLOBAL)
    }

    /// Load a shared library with custom flags.
    fn load_with_flags(path: &str, flags: ::std::os::raw::c_int) -> Result<Self, ()> {
        let path = CString::new(path).unwrap();

        // Open the shared library with the specified flags
        let module = unsafe { dlopen(path.as_ptr(), flags) };
        if module.is_null() {
            eprintln!("Failed to load module: {}", path.to_string_lossy());
            Err(())
        } else {
            Ok(Module(unsafe { NonNull::new_unchecked(module) }))
        }
    }

    /// Get a symbol from the loaded module.
    pub fn get_symbol<F: Sized>(&self, name: &str) -> Result<F, ()> {
        let name = CString::new(name).unwrap();
        let symbol = unsafe { dlsym(self.0.as_ptr(), name.as_ptr()) };

        if symbol.is_null() {
            eprintln!("Failed to find symbol: {}", name.to_string_lossy());
            return Err(());
        }

        // Transmute the symbol to the desired type.
        Ok(unsafe { std::mem::transmute_copy::<_, F>(&symbol) })
    }
}

impl Drop for Module {
    fn drop(&mut self) {
        // When the module loader is dropped, close the shared library.
        unsafe { dlclose(self.0.as_ptr()) };
    }
}

#[macro_export]
macro_rules! declare_module {
    ($name:ident,
    $path:literal$(, $fallback:literal)*$(,)?
    ...
    // static
    $($s_vis:vis $s_name:ident: $s_type:ty,)*
    ...
    // function
    $($f_vis:vis fn $f_name:ident($($f_arg:ty),*$(,)?)$( -> $f_ret:ty )?,)*
    ...
    // function with variadic arguments
    $($v_vis:vis fn $v_name:ident($($v_arg:ty),*,...)$( -> $v_ret:ty )?,)*
    ...
    // extra field (should be `Default` so `try_load` can construct it)
    $($vis:vis $field:ident: $field_ty:ty,)*) => {
        #[derive(Clone)]
        pub struct $name {
            _module: std::rc::Rc<$crate::module::Module>,
            $($s_vis $s_name: $s_type,)*
            $($f_vis $f_name: unsafe extern "C" fn ($($f_arg),*)$( -> $f_ret)?,)*
            $($v_vis $v_name: unsafe extern "C" fn ($($v_arg),*, ...)$( -> $v_ret)?,)*
            $($vis $field: $field_ty)*
        }
        impl $name {
            pub fn try_load() -> Result<Self, ()> {
                $crate::module::Module::load($path)$(.or_else(|_| $crate::module::Module::load($fallback)))*
                .map(|module|
                    $name {
                        $($s_name: module.get_symbol::<$s_type>(stringify!($s_name)).unwrap(),)*
                        $($f_name: module.get_symbol::<unsafe extern "C" fn ($($f_arg),*)$( -> $f_ret)?>(stringify!($f_name)).unwrap(),)*
                        $($v_name: module.get_symbol::<unsafe extern "C" fn ($($v_arg),*, ...)$( -> $v_ret)?>(stringify!($v_name)).unwrap(),)*
                        $($field: Default::default(),)*
                        _module: module.into(),
                    }
                )
            }
        }
    }
}
