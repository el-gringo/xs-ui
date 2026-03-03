use xs_js_sys::{fetch, log};

//#[unsafe(no_mangle)]
pub fn main() {
    fetch("https://httpbin.io/status/418".into(), None, |response| {
       log(&response);
    });
}
