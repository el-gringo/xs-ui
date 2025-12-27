#![allow(dead_code)]

use xs_backend::Backend;

mod app;

//mod with_rsx;
mod with_builder;
pub fn main() {
    with_builder::main();
}
