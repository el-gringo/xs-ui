extern crate proc_macro;
use proc_macro::{Delimiter, Ident, Punct, Spacing, TokenStream, TokenTree, token_stream::IntoIter};
use std::str::FromStr;

use crate::generator::Tokenizer;


#[derive(Debug)]
struct Prop {
    name: Ident,
    ty: TokenStream,
    //default: Option<Value>,
}

impl Tokenizer for Vec<Prop> {
    fn tokenize(self) -> TokenStream {
        self.iter()
            .map(|Prop { name, ty }| {
                let mut stream = TokenStream::new();
                stream.extend(vec![Ident::new("pub", name.span())]);
                stream.extend(vec![name.clone()]);
                stream.extend(vec![Punct::new(':', Spacing::Alone)]);
                stream.extend(ty.clone());
                stream.extend(vec![Punct::new(',', Spacing::Alone)]);
                stream
            })
            .collect()
    }
}

#[derive(Debug)]
struct Component {
    name: String,
    props: Vec<Prop>,
    body: TokenStream,
    return_type: TokenStream
}

impl Component {
    pub fn new(name: String, props: Vec<Prop>, body: TokenStream, return_type: TokenStream) -> Self {
        Self { name, props, body, return_type }
    }
}

fn visit_function_argument(iter: &mut IntoIter) -> Result<Prop, ()> {
    let mut name: Option<Ident> = None;
    let mut ty = TokenStream::new();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => {
                name = Some(ident);
            }
            TokenTree::Punct(punct) if punct.as_char() == ':' => {
                let mut stream: Vec<TokenTree> = Vec::new();
                for token in iter.by_ref() {
                    if let TokenTree::Punct(punct) = &token
                        && punct.as_char() == ','
                    {
                        break;
                    } else {
                        stream.push(token);
                    }
                }
                ty.extend(stream);
                break;
            }
            _ => {}
        }
    }
    if let Some(name) = name {
        return Ok(Prop { name, ty });
    }
    Err(())
}

fn visit_function_arguments(iter: &mut IntoIter) -> Vec<Prop> {
    let mut props: Vec<Prop> = Vec::new();
    if let Some(token) = iter.next()
        && let TokenTree::Group(group) = token
    {
        let mut iter = group.stream().into_iter();
        while let Ok(prop) = visit_function_argument(&mut iter) {
            props.push(prop);
        }
        return props;
    }
    if let Some(_token) = iter.next() {
        // return value
    }
    props
}

fn visit_function_return_type(iter: &mut IntoIter) -> (TokenStream, TokenStream) {
    let mut is_return = false;
    let mut stream = TokenStream::new();
    while let Some(token) = iter.next() {
        if let TokenTree::Punct(punct) = token {
            if is_return && punct.as_char() == '>' {
                for token in iter.by_ref() {
                    match token {
                        TokenTree::Group(group) => {
                            if group.delimiter() == Delimiter::Brace {
                                return (group.stream(), stream);
                            }
                        },
                        _ => {
                            stream.extend(vec![token]);
                        }
                    }
                }
            }
            if punct.as_char() == '-' {
                is_return = true;
            }
        }
    }
    (TokenStream::new(), TokenStream::new())
}

fn visit_function(iter: &mut IntoIter) -> Result<Component, ()> {
    let mut is_function_name = false;
    while let Some(token) = iter.next() {
        if let TokenTree::Ident(ident) = token {
            if is_function_name {
                let arguments = visit_function_arguments(iter);
                let (body, return_type) = visit_function_return_type(iter);
                return Ok(Component::new(
                    ident.to_string(),
                    arguments,
                    body,
                    return_type,
                ));
            } else {
                if ident.to_string() == "fn" {
                    is_function_name = true;
                }
            }
        }
    }
    Err(())
}

pub fn parse_component(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    if let Ok(component) = visit_function(&mut iter) {
        let name = component.name.to_string();
        let body = component.body;
        let return_type = component.return_type;

        let props: Vec<(String, TokenStream)> = component
            .props
            .into_iter()
            .map(|prop| (prop.name.to_string(), prop.ty))
            .collect();
        let generics = props
            .clone()
            .into_iter()
            .map(|(name, _)| format!("{name}=Undefined"))
            .collect::<Vec<String>>()
            .join(", ");
        let struct_fields = props
            .clone()
            .into_iter()
            .map(|(name, _)| format!("pub {name}: {name}"))
            .collect::<Vec<String>>()
            .join(", ");
        let destructured_props = props
            .clone()
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<String>>()
            .join(", ");
        let prop_values = props
            .clone()
            .into_iter()
            .map(|(_, value)| value.to_string())
            .collect::<Vec<String>>()
            .join(", ");
        let code = format!(
            "
            pub mod {name} {{
                pub use super::*;
                pub use xs_core::Undefined;

                pub struct Props<{generics}> {{
                    {struct_fields}
                }}

                pub fn render(Props {{ {destructured_props} }}: Props<{prop_values}>) -> {return_type} {{
                    {body}
                }}
            }}
        "
        );
        if let Ok(output) = TokenStream::from_str(&code) {
            return output;
        }
    }
    TokenStream::new()
}
