extern crate proc_macro;
use proc_macro::TokenStream;

mod component;
mod generator;
mod rsx;
//mod rsx2;
mod prop;

use component::parse_component;
use rsx::parse_rsx;
use prop::parse_prop;

#[proc_macro_attribute]
pub fn component(_attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_component(input)
}

#[proc_macro]
pub fn rsx(input: TokenStream) -> TokenStream {
    parse_rsx(input)
}

#[proc_macro_attribute]
pub fn prop(attr: TokenStream, input: TokenStream) -> TokenStream {
    parse_prop(attr, input)
}
