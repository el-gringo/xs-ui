use std::marker::PhantomData;

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn heap_drop(idx: u32);
    }
}

pub struct JsValue {
    pub idx: u32,
    _marker: PhantomData<*mut u8>,
}

impl JsValue {
    pub fn new(idx: u32) -> Self {
        Self {
            idx,
            _marker: PhantomData,
        }
    }
}

impl Drop for JsValue {
    fn drop(&mut self) {
        unsafe { js::heap_drop(self.idx) }
    }
}

pub struct Object<T = JsValue> {
    _obj: JsValue,
    generics: PhantomData<(T,)>,
}
