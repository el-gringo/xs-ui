use std::ops::Deref;

use crate::core::JsValue;

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn element_query_selector(
            element_ptr: u32,
            selector_ptr: *const u8,
            selector_len: usize,
        ) -> u32;

        pub fn element_offset_width(element_ptr: u32) -> u32;
        pub fn element_offset_height(element_ptr: u32) -> u32;
    }
}

pub struct HTMLElement(JsValue);

impl HTMLElement {
    pub fn from_js(js: JsValue) -> Self {
        Self(js)
    }
}

impl Deref for HTMLElement {
    type Target = JsValue;

    fn deref(&self) -> &JsValue {
        &self.0
    }
}

pub trait Node {
    fn query_selector(&self, selector: &str) -> HTMLElement;
    fn offset_width(&self) -> u32;
    fn offset_height(&self) -> u32;
}

impl Node for JsValue {
    fn query_selector(&self, selector: &str) -> HTMLElement {
        unsafe {
            let ptr = js::element_query_selector(self.idx, selector.as_ptr(), selector.len());
            HTMLElement::from_js(JsValue::new(ptr))
        }
    }

    fn offset_width(&self) -> u32 {
        unsafe { js::element_offset_width(self.idx) }
    }

    fn offset_height(&self) -> u32 {
        unsafe { js::element_offset_height(self.idx) }
    }
}
