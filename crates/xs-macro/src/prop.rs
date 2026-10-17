use proc_macro::{TokenStream, TokenTree};

pub fn parse_prop(attr: TokenStream, _input: TokenStream) -> TokenStream {
    //let mut is_attribute_value = false;
    let output = TokenStream::new();
    for token in attr {
        match token {
            TokenTree::Ident(ident) => {
                if ident.to_string() == "default" {

                }
            },
            TokenTree::Punct(punct) if punct.as_char() == '=' => {
                //is_attribute_value = true;
            },
            _ => {

            }
        }
    }
    output
}
