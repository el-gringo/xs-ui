use std::ops::Deref;

use crate::core::JsValue;

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn document() -> u32;
    }
}

pub struct Document(JsValue);

impl Document {
    pub fn from_js(js: JsValue) -> Self {
        Self(js)
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
