use crate::{AnyClosure, Closure, core::JsValue, invoke_closure, store_closure};

#[repr(C)]
struct EventKeyDownRaw {
    key_code: u32,
    modifiers: u32,
    key_ptr: u32,
    key_len: u32,
}

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn add_event_listener(
            obj_ptr: u32,
            event_name_ptr: *const u8,
            event_name_len: usize,
            closure_handle: usize,
        );
        pub fn event_key_down(event_ptr: u32, buf: *mut u8);
    }
}

#[allow(dead_code)]
pub struct Event(JsValue);

pub trait EventElement: Sized {
    fn from_js(js: JsValue) -> Self;
}

impl EventElement for Event {
    fn from_js(js: JsValue) -> Self {
        Event(js)
    }
}

pub struct EventResize;

impl EventElement for EventResize {
    fn from_js(_js: JsValue) -> Self {
        EventResize
    }
}

pub struct EventKeyDown {
    key_code: u32,
    modifiers: u8,
    key: String,
}

impl EventKeyDown {
    pub fn key_code(&self) -> u32 {
        self.key_code
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn shift(&self) -> bool {
        self.modifiers & 1 != 0
    }

    pub fn ctrl(&self) -> bool {
        self.modifiers & 2 != 0
    }

    pub fn alt(&self) -> bool {
        self.modifiers & 4 != 0
    }

    pub fn meta(&self) -> bool {
        self.modifiers & 8 != 0
    }
}

impl EventElement for EventKeyDown {
    fn from_js(js: JsValue) -> Self {
        let mut raw = EventKeyDownRaw {
            key_code: 0,
            modifiers: 0,
            key_ptr: 0,
            key_len: 0,
        };
        unsafe {
            js::event_key_down(js.idx, &mut raw as *mut EventKeyDownRaw as *mut u8);
            let key = {
                let slice =
                    std::slice::from_raw_parts(raw.key_ptr as *const u8, raw.key_len as usize);
                String::from_utf8_unchecked(slice.to_vec())
            };
            EventKeyDown {
                key_code: raw.key_code,
                modifiers: raw.modifiers as u8,
                key,
            }
        }
    }
}

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
        let mut handler = event_handler;
        let wrapper: Closure<JsValue> = Closure::new(move |js: JsValue| {
            let event = E::from_js(js);
            handler.call(event);
        });
        unsafe {
            let closure_handle = store_closure(AnyClosure::new(wrapper));
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
