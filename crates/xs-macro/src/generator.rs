use proc_macro::TokenStream;

#[allow(dead_code)]
pub trait Tokenizer {
    fn tokenize(self) -> TokenStream;
}
