use xs_js_sys::{Document, document};

pub fn main() {
    let document = document();
    document.query_selector("#canvas");
}
