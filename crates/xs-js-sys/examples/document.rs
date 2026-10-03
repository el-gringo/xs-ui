use xs_js_sys::{Element, Node, document, log};

fn main() {
    let document = document();
    let body = document.query_selector("body");
    body.set_style("background", "#0f172a");
    body.set_style("color", "#e2e8f0");
    log("document example: styled the body");
}
