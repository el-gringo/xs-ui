use crate::{
    AnyClosure, Closure,
    core::{JsValue, Object},
    invoke_closure, store_closure,
};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn add_event_listener(
            obj_ptr: u32,
            event_name_ptr: *const u8,
            event_name_len: usize,
            closure_handle: usize,
        );
    }
}

pub struct Event {
    _obj: Object,
}
pub trait EventElement: Sized {}
impl EventElement for Event {}

pub struct EventResize;
impl EventElement for EventResize {}

pub trait EventEmitter: Sized {
    fn add_event_listener<E>(&self, event_name: &'static str, event_handler: Closure<E>)
    where
        E: EventElement;
}

impl EventEmitter for JsValue {
    fn add_event_listener<E>(&self, event_name: &'static str, event_handler: Closure<E>)
    where
        E: EventElement,
    {
        unsafe {
            let closure_handle = store_closure(AnyClosure::new(event_handler));
            js::add_event_listener(
                self.idx,
                event_name.as_ptr(),
                event_name.len(),
                closure_handle,
            );
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn callback_event_handler(closure_handle: usize, event_ptr: u32) {
    invoke_closure(closure_handle, JsValue::new(event_ptr));
}
