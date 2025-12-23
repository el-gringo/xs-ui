use proc_macro::{TokenStream, TokenTree};

pub fn parse_prop(attr: TokenStream, input: TokenStream) -> TokenStream {
    let mut is_attribute_value = false;
    let mut output = TokenStream::new();
    let mut iter = attr.into_iter();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => {
                if ident.to_string() == "default" {

                }
            },
            TokenTree::Punct(punct) => {
                if punct.as_char() == '=' {
                    is_attribute_value = true;
                }
            },
            _ => {

            }
        }
    }
    output
}
