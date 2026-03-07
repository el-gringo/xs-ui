use std::{cell::RefCell, rc::Rc};

use xs_js_sys::{Closure, fetch, log};

pub fn main() {
    fetch("https://httpbin.io/status/418".into(), None, Closure::new(|response: Vec<u8>| {
        log("response");
        let body = String::from_utf8(response).unwrap();
        log("response");
        log(body.as_str());
    }));

}
