extern crate proc_macro;
use crate::generator::Tokenizer;
use proc_macro::{Ident, Punct, Spacing, TokenStream, TokenTree, token_stream::IntoIter};
use std::str::FromStr;

#[derive(Debug)]
struct Prop {
    name: Ident,
    ty: TokenStream,
    //default: Option<Value>,
}

impl Tokenizer for Vec<Prop> {
    fn tokenize(&mut self) -> TokenStream {
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
}

impl Component {
    pub fn new(name: String, props: Vec<Prop>, body: TokenStream) -> Self {
        Self { name, props, body }
    }
}

fn visit_function_argument(iter: &mut IntoIter) -> Result<Prop, ()> {
    let mut name: Option<Ident> = None;
    let mut is_argument_value = false;
    let mut ty = TokenStream::new();
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => {
                if !is_argument_value {
                    name = Some(ident);
                }
            }
            TokenTree::Punct(punct) => {
                if is_argument_value && punct.as_char() == ',' {
                    break;
                }
                if punct.as_char() == ':' {
                    is_argument_value = true;
                    let mut stream: Vec<TokenTree> = Vec::new();
                    while let Some(token) = iter.next() {
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
    if let Some(token) = iter.next() {
        if let TokenTree::Group(group) = token {
            let mut iter = group.stream().into_iter();
            while let Ok(prop) = visit_function_argument(&mut iter) {
                props.push(prop);
            }
            return props;
        }
    }
    if let Some(token) = iter.next() {
        // return value
    }
    return props;
}

fn visit_function_body(iter: &mut IntoIter) -> TokenStream {
    while let Some(token) = iter.next() {
        if let TokenTree::Group(group) = token {
            return group.stream();
        }
    }
    TokenStream::new()
}

fn visit_function(iter: &mut IntoIter) -> Result<Component, ()> {
    let mut is_function_name = false;
    while let Some(token) = iter.next() {
        if let TokenTree::Ident(ident) = token {
            if is_function_name {
                return Ok(Component::new(
                    ident.to_string(),
                    visit_function_arguments(iter),
                    visit_function_body(iter),
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
            .map(|(name, _)| format!("pub  {name}: {name}"))
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

                pub fn render(Props {{ {destructured_props} }}: Props<{prop_values}>) -> Element {{
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
