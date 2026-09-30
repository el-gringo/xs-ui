use crate::{
    AnyClosure, Closure, core::JsValue, element::HTMLElement, invoke_closure, store_closure,
};

/// The layout has to match `event_key_down` in `bindings/event.js`.
#[repr(C)]
struct EventKeyDownRaw {
    modifiers: u32,
    code_ptr: u32,
    code_len: u32,
    key_code: u32,
}

/// Mirrors `MouseEvent`. The layout has to match `event_mouse` in
/// `bindings/event.js`.
#[repr(C)]
struct EventMouseRaw {
    client_x: f64,
    client_y: f64,
    offset_x: f64,
    offset_y: f64,
    button: u16,
    buttons: u16,
    modifiers: u8,
    padding: [u8; 3],
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
        pub fn event_mouse(event_ptr: u32, buf: *mut u8);
        pub fn event_target(event_ptr: u32) -> u32;
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
    modifiers: u8,
    /// The physical key, layout agnostic.
    code: String,
    /// The deprecated `KeyboardEvent.keyCode`.
    key_code: u32,
}

impl EventKeyDown {
    /// `KeyboardEvent.code`, what the key *is* rather than the character it
    /// produces, so it does not change with the keyboard layout: `ArrowUp`,
    /// `KeyA`, `Enter`, `Digit1`, `NumpadEnter`...
    pub fn code(&self) -> &str {
        &self.code
    }

    /// The deprecated `KeyboardEvent.keyCode`, a layout dependent legacy number.
    /// Prefer [`Self::code`].
    pub fn key_code(&self) -> u32 {
        self.key_code
    }

    pub fn modifiers(&self) -> u8 {
        self.modifiers
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
            modifiers: 0,
            code_ptr: 0,
            code_len: 0,
            key_code: 0,
        };
        unsafe {
            js::event_key_down(js.idx, &mut raw as *mut EventKeyDownRaw as *mut u8);
            let code = {
                let slice =
                    std::slice::from_raw_parts(raw.code_ptr as *const u8, raw.code_len as usize);
                String::from_utf8_unchecked(slice.to_vec())
            };
            EventKeyDown {
                modifiers: raw.modifiers as u8,
                code,
                key_code: raw.key_code,
            }
        }
    }
}

/// A `MouseEvent`: `mousedown`, `mousemove`, `mouseup`, `click`, ...
pub struct EventMouse {
    /// Keeps the js event alive: the JsValue it is built from owns a heap slot
    /// and frees it on drop.
    event: JsValue,
    client_x: f64,
    client_y: f64,
    offset_x: f64,
    offset_y: f64,
    button: u16,
    buttons: u16,
    modifiers: u8,
}

impl EventMouse {
    /// Position in the viewport, css pixels.
    pub fn client_x(&self) -> f64 {
        self.client_x
    }
    pub fn client_y(&self) -> f64 {
        self.client_y
    }
    /// Position relative to the padding box of the target.
    pub fn offset_x(&self) -> f64 {
        self.offset_x
    }
    pub fn offset_y(&self) -> f64 {
        self.offset_y
    }
    /// 0 main (left), 1 auxiliary (middle), 2 secondary (right).
    pub fn button(&self) -> u16 {
        self.button
    }
    /// Bit field of the buttons held down: 1 main, 2 secondary, 4 auxiliary.
    pub fn buttons(&self) -> u16 {
        self.buttons
    }
    pub fn modifiers(&self) -> u8 {
        self.modifiers
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
    /// The element the event fired on, for hit testing.
    pub fn target(&self) -> HTMLElement {
        unsafe { HTMLElement::from_js(JsValue::new(js::event_target(self.event.idx))) }
    }
}

impl EventElement for EventMouse {
    fn from_js(js: JsValue) -> Self {
        let mut raw = EventMouseRaw {
            client_x: 0.,
            client_y: 0.,
            offset_x: 0.,
            offset_y: 0.,
            button: 0,
            buttons: 0,
            modifiers: 0,
            padding: [0; 3],
        };
        unsafe {
            js::event_mouse(js.idx, &mut raw as *mut EventMouseRaw as *mut u8);
            EventMouse {
                event: js,
                client_x: raw.client_x,
                client_y: raw.client_y,
                offset_x: raw.offset_x,
                offset_y: raw.offset_y,
                button: raw.button,
                buttons: raw.buttons,
                modifiers: raw.modifiers,
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
