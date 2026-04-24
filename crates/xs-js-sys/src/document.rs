use crate::core::JsValue;

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn document() -> u32;
        pub fn document_query_selector(
            document_ptr: u32,
            selector_ptr: *const u8,
            selector_len: usize,
        ) -> u32;
    }
}

pub trait Document: Sized {
    fn query_selector(&self, selector: &'static str) -> JsValue;
}

impl Document for JsValue {
    fn query_selector(&self, selector: &'static str) -> JsValue {
        unsafe {
            let ptr = js::document_query_selector(self.idx, selector.as_ptr(), selector.len() as _);
            JsValue::new(ptr)
        }
    }
}

pub fn document() -> impl Document {
    unsafe {
        let ptr = js::document();
        JsValue::new(ptr)
    }
}
