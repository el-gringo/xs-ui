use crate::{EventEmitter, core::JsValue};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn canvas_get_context(
            canvas_ptr: u32,
            context_type_ptr: *const u8,
            context_type_len: usize,
        ) -> u32;
        pub fn canvas_get_width(canvas_ptr: u32) -> u32;
        pub fn canvas_get_height(canvas_ptr: u32) -> u32;
        pub fn canvas_set_width(canvas_ptr: u32, width: u32);
        pub fn canvas_set_height(canvas_ptr: u32, height: u32);
    }
}

/// Mirrors the `HTMLCanvasElement` properties.
pub trait Canvas: Sized + EventEmitter {
    fn get_context(&self, context: &'static str) -> JsValue;

    /// Size of the drawing buffer, in pixels. Setting it resizes the buffer and
    /// clears it. It is independent of the css size, see `offset_width`.
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn set_width(&self, width: u32);
    fn set_height(&self, height: u32);
}

impl Canvas for JsValue {
    fn get_context(&self, context_type: &'static str) -> JsValue {
        unsafe {
            let idx = js::canvas_get_context(self.idx, context_type.as_ptr(), context_type.len());
            JsValue::new(idx)
        }
    }

    fn width(&self) -> u32 {
        unsafe { js::canvas_get_width(self.idx) }
    }

    fn height(&self) -> u32 {
        unsafe { js::canvas_get_height(self.idx) }
    }

    fn set_width(&self, width: u32) {
        unsafe {
            js::canvas_set_width(self.idx, width);
        }
    }

    fn set_height(&self, height: u32) {
        unsafe {
            js::canvas_set_height(self.idx, height);
        }
    }
}
