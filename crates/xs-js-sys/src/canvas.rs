use crate::{EventEmitter, core::JsValue};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn canvas_get_context(
            canvas_ptr: u32,
            context_type_ptr: *const u8,
            context_type_len: usize,
        ) -> u32;
        pub fn canvas_width(canvas_ptr: u32) -> u32;
        pub fn canvas_height(canvas_ptr: u32) -> u32;
    }
}

pub trait Canvas: Sized + EventEmitter {
    fn get_context(&self, context: &'static str) -> JsValue;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
}

impl Canvas for JsValue {
    fn get_context(&self, context_type: &'static str) -> JsValue {
        unsafe {
            let idx = js::canvas_get_context(self.idx, context_type.as_ptr(), context_type.len());
            JsValue::new(idx)
        }
    }

    fn width(&self) -> u32 {
        unsafe { js::canvas_width(self.idx) }
    }

    fn height(&self) -> u32 {
        unsafe { js::canvas_height(self.idx) }
    }
}
