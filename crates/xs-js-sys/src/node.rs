use crate::{core::JsValue, element::HTMLElement};

mod js {
    #[link(wasm_import_module = "env")]
    unsafe extern "C" {
        pub fn node_query_selector(
            element_ptr: u32,
            selector_ptr: *const u8,
            selector_len: usize,
        ) -> u32;

        pub fn node_append_child(parent_ptr: u32, child_ptr: u32);
        pub fn node_insert_before(parent_ptr: u32, node_ptr: u32, reference_ptr: u32);
        pub fn node_remove_child(parent_ptr: u32, child_ptr: u32);
        pub fn node_replace_child(parent_ptr: u32, new_child_ptr: u32, old_child_ptr: u32);
    }
}

pub trait Node {
    /// Looks a descendant up with a css selector.
    fn query_selector(&self, selector: &str) -> HTMLElement;

    /// Adds `child` as the last child, moving it if it is already attached.
    fn append_child(&self, child: &HTMLElement);
    /// Inserts `node` before `reference`, or as the last child when
    /// `reference` is not a child of this node.
    fn insert_before(&self, node: &HTMLElement, reference: &HTMLElement);
    /// Detaches `child`, which must be a child of this node.
    fn remove_child(&self, child: &HTMLElement);
    /// Replaces `old_child` with `new_child` and returns `old_child`.
    fn replace_child(&self, new_child: &HTMLElement, old_child: &HTMLElement);
}

impl Node for JsValue {
    fn query_selector(&self, selector: &str) -> HTMLElement {
        unsafe {
            let ptr = js::node_query_selector(self.idx, selector.as_ptr(), selector.len());
            HTMLElement::from_js(JsValue::new(ptr))
        }
    }

    fn append_child(&self, child: &HTMLElement) {
        unsafe {
            js::node_append_child(self.idx, child.idx);
        }
    }

    fn insert_before(&self, node: &HTMLElement, reference: &HTMLElement) {
        unsafe {
            js::node_insert_before(self.idx, node.idx, reference.idx);
        }
    }

    fn remove_child(&self, child: &HTMLElement) {
        unsafe {
            js::node_remove_child(self.idx, child.idx);
        }
    }

    fn replace_child(&self, new_child: &HTMLElement, old_child: &HTMLElement) {
        unsafe {
            js::node_replace_child(self.idx, new_child.idx, old_child.idx);
        }
    }
}
