use std::ops::Deref;

use crate::{core::JsValue, element::HTMLElement};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn document() -> u32;
        pub fn document_create_element(tag_ptr: u32, tag_len: u32) -> u32;
    }
}

pub struct Document(JsValue);

impl Document {
    pub fn from_js(js: JsValue) -> Self {
        Self(js)
    }

    /// Creates a detached element, e.g. `create_element("div")`.
    pub fn create_element(&self, tag: &str) -> HTMLElement {
        unsafe {
            let ptr = js::document_create_element(tag.as_ptr() as u32, tag.len() as u32);
            HTMLElement::from_js(JsValue::new(ptr))
        }
    }
}

impl Deref for Document {
    type Target = JsValue;

    fn deref(&self) -> &JsValue {
        &self.0
    }
}

pub fn document() -> Document {
    unsafe {
        let ptr = js::document();
        Document(JsValue::new(ptr))
    }
}
