use std::ops::Deref;

use crate::core::JsValue;

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn element_offset_width(element_ptr: u32) -> u32;
        pub fn element_offset_height(element_ptr: u32) -> u32;

        pub fn element_inner_html(element_ptr: u32) -> u32;
        pub fn element_set_inner_html(element_ptr: u32, html_ptr: *const u8, html_len: usize);

        /// `-1` when the attribute is not set.
        pub fn element_attribute(element_ptr: u32, name_ptr: *const u8, name_len: usize) -> i32;
        pub fn element_set_attribute(
            element_ptr: u32,
            name_ptr: *const u8,
            name_len: usize,
            value_ptr: *const u8,
            value_len: usize,
        );
        pub fn element_remove_attribute(element_ptr: u32, name_ptr: *const u8, name_len: usize);

        /// Empty string when the property is not set.
        pub fn element_style(element_ptr: u32, property_ptr: *const u8, property_len: usize)
        -> u32;
        pub fn element_set_style(
            element_ptr: u32,
            property_ptr: *const u8,
            property_len: usize,
            value_ptr: *const u8,
            value_len: usize,
        );

        pub fn element_remove(element_ptr: u32);
        pub fn element_focus(element_ptr: u32);

        pub fn element_value(element_ptr: u32) -> u32;
        pub fn element_set_value(element_ptr: u32, value_ptr: *const u8, value_len: usize);

        pub fn element_class_name(element_ptr: u32) -> u32;
        pub fn element_set_class_name(element_ptr: u32, name_ptr: *const u8, name_len: usize);

        pub fn element_class_list(element_ptr: u32) -> u32;

        pub fn class_list_add(list_ptr: u32, token_ptr: *const u8, token_len: usize);
        pub fn class_list_remove(list_ptr: u32, token_ptr: *const u8, token_len: usize);
        /// `1` when the token is present once toggled.
        pub fn class_list_toggle(list_ptr: u32, token_ptr: *const u8, token_len: usize) -> u32;
        pub fn class_list_contains(list_ptr: u32, token_ptr: *const u8, token_len: usize) -> u32;
    }
}

/// An element, as returned by a query. Derefs to the raw js value, so it also
/// gets the [`Node`] and [`TokenList`] methods.
///
/// [`Node`]: crate::Node
/// [`TokenList`]: crate::TokenList
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

/// The element's `classList`, a `DOMTokenList`.
pub struct ClassList(JsValue);

impl ClassList {
    pub fn from_js(js: JsValue) -> Self {
        Self(js)
    }
}

impl Deref for ClassList {
    type Target = JsValue;

    fn deref(&self) -> &JsValue {
        &self.0
    }
}

/// Mirrors `DOMTokenList`.
pub trait TokenList {
    fn add(&self, token: &str);
    fn remove(&self, token: &str);
    /// Adds the token when it is missing, removes it when it is there.
    /// Returns whether it is there afterwards.
    fn toggle(&self, token: &str) -> bool;
    fn contains(&self, token: &str) -> bool;
}

impl TokenList for JsValue {
    fn add(&self, token: &str) {
        unsafe {
            js::class_list_add(self.idx, token.as_ptr(), token.len());
        }
    }

    fn remove(&self, token: &str) {
        unsafe {
            js::class_list_remove(self.idx, token.as_ptr(), token.len());
        }
    }

    fn toggle(&self, token: &str) -> bool {
        unsafe { js::class_list_toggle(self.idx, token.as_ptr(), token.len()) != 0 }
    }

    fn contains(&self, token: &str) -> bool {
        unsafe { js::class_list_contains(self.idx, token.as_ptr(), token.len()) != 0 }
    }
}

/// Mirrors `HTMLElement`.
///
/// Getters have no prefix, setters are `set_` prefixed. Every string they
/// return is copied out of the js value, hence `String` and not `&str`.
pub trait Element {
    fn offset_width(&self) -> u32;
    fn offset_height(&self) -> u32;

    /// Replaces the children with parsed html.
    fn set_inner_html(&self, html: &str);
    fn inner_html(&self) -> String;

    /// `None` when the attribute is not set.
    fn attribute(&self, name: &str) -> Option<String>;
    fn set_attribute(&self, name: &str, value: &str);
    fn remove_attribute(&self, name: &str);

    /// Empty string when the property is not set.
    fn style(&self, property: &str) -> String;
    /// Sets an inline style property, e.g. `("background-color", "red")`.
    fn set_style(&self, property: &str, value: &str);

    /// The `value` of a form control, empty string when it has none.
    fn value(&self) -> String;
    fn set_value(&self, value: &str);

    /// The class attribute as a whole, space separated.
    fn class_name(&self) -> String;
    /// Replaces every class, like the `className` property does.
    fn set_class_name(&self, name: &str);

    fn class_list(&self) -> ClassList;

    /// Detaches the element from its parent.
    fn remove(&self);
    fn focus(&self);
}

/// Reads a string the bindings wrote in wasm memory as a null terminated c
/// string. The buffer belongs to the heap allocator and is not freed.
unsafe fn read_c_string(ptr: u32) -> String {
    unsafe {
        std::ffi::CStr::from_ptr(ptr as *const std::ffi::c_char)
            .to_string_lossy()
            .into_owned()
    }
}

impl Element for JsValue {
    fn offset_width(&self) -> u32 {
        unsafe { js::element_offset_width(self.idx) }
    }

    fn offset_height(&self) -> u32 {
        unsafe { js::element_offset_height(self.idx) }
    }

    fn set_inner_html(&self, html: &str) {
        unsafe {
            js::element_set_inner_html(self.idx, html.as_ptr(), html.len());
        }
    }

    fn inner_html(&self) -> String {
        unsafe { read_c_string(js::element_inner_html(self.idx)) }
    }

    fn attribute(&self, name: &str) -> Option<String> {
        unsafe {
            let ptr = js::element_attribute(self.idx, name.as_ptr(), name.len());
            (ptr >= 0).then(|| read_c_string(ptr as u32))
        }
    }

    fn set_attribute(&self, name: &str, value: &str) {
        unsafe {
            js::element_set_attribute(
                self.idx,
                name.as_ptr(),
                name.len(),
                value.as_ptr(),
                value.len(),
            );
        }
    }

    fn remove_attribute(&self, name: &str) {
        unsafe {
            js::element_remove_attribute(self.idx, name.as_ptr(), name.len());
        }
    }

    fn style(&self, property: &str) -> String {
        unsafe {
            read_c_string(js::element_style(
                self.idx,
                property.as_ptr(),
                property.len(),
            ))
        }
    }

    fn set_style(&self, property: &str, value: &str) {
        unsafe {
            js::element_set_style(
                self.idx,
                property.as_ptr(),
                property.len(),
                value.as_ptr(),
                value.len(),
            );
        }
    }

    fn value(&self) -> String {
        unsafe { read_c_string(js::element_value(self.idx)) }
    }

    fn set_value(&self, value: &str) {
        unsafe {
            js::element_set_value(self.idx, value.as_ptr(), value.len());
        }
    }

    fn class_name(&self) -> String {
        unsafe { read_c_string(js::element_class_name(self.idx)) }
    }

    fn set_class_name(&self, name: &str) {
        unsafe {
            js::element_set_class_name(self.idx, name.as_ptr(), name.len());
        }
    }

    fn class_list(&self) -> ClassList {
        unsafe { ClassList::from_js(JsValue::new(js::element_class_list(self.idx))) }
    }

    fn remove(&self) {
        unsafe {
            js::element_remove(self.idx);
        }
    }

    fn focus(&self) {
        unsafe {
            js::element_focus(self.idx);
        }
    }
}
