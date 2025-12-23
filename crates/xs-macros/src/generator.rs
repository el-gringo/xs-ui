use proc_macro::TokenStream;



pub trait Tokenizer {
    fn tokenize(&mut self) -> TokenStream;
}
