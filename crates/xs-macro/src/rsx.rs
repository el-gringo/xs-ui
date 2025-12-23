extern crate proc_macro;
use proc_macro::{TokenStream, TokenTree, token_stream::IntoIter};
use std::{collections::HashMap, str::FromStr};

type Attributes = HashMap<String, TokenStream>;
#[derive(Debug)]
struct Node {
    name: String,
    attributes: Attributes,
    children: Vec<Node>,
}

impl Node {
    pub fn tokenize(&mut self) -> TokenStream {
        let mut output = TokenStream::new();
        let name = &self.name;
        let children = self
            .children
            .iter_mut()
            .map(|node| node.tokenize().to_string())
            .collect::<Vec<String>>()
            .join(", ");
        self.attributes
            .insert(
                "children".to_string(),
                TokenStream::from_str(
                    format!("vec![{children}]").as_str()
                ).unwrap()
            );
        let props = self
            .attributes
            .iter()
            .map(|(name, value)| format!("{name}: {value}"))
            .collect::<Vec<String>>()
            .join(", ");
        let code = format!("{name}::render({name}::Props {{ {props} }})");
        if let Ok(stream) = TokenStream::from_str(&code) {
            output.extend(stream);
        }
        output
    }
}

pub fn parse_rsx(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    visit_tag(&mut iter).unwrap().tokenize()
}

fn visit_tag(iter: &mut IntoIter) -> Result<Node, &'static str> {
    let mut is_opening_tag = false;
    let mut is_closing_tag = false;
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '<' {
                    is_opening_tag = true;
                }
                if is_opening_tag && punct.as_char() == '/' {
                    is_opening_tag = false;
                    is_closing_tag = true;
                }
            }
            TokenTree::Ident(ident) => {
                if is_opening_tag {
                    let name = ident.to_string();
                    let attributes = visit_attibutes(iter);
                    let mut children: Vec<Node> = Vec::new();
                    while let Ok(child) = visit_tag(iter) {
                        children.push(child);
                    }
                    return Ok(Node {
                        name,
                        attributes,
                        children,
                    });
                }
                if is_closing_tag {
                    break;
                }
            }
            _ => {}
        }
    }
    Err("failed parsing tag")
}

fn visit_attibute(iter: &mut IntoIter) -> Option<(String, TokenStream)> {
    let mut name: Option<String> = None;
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Ident(ident) => {
                name = Some(ident.to_string());
            }
            TokenTree::Literal(literal) => {
                let mut value = TokenStream::new();
                value.extend(vec![ literal ]);
                if let Some(name) = name {
                    return Some((name, value));
                }
            }
            TokenTree::Group(group) => {
                let value = group.stream();
                if let Some(name) = name {
                    return Some((name, value));
                }
            }
            TokenTree::Punct(punct) => {
                if punct.as_char() == '=' {
                    continue;
                }
                if punct.as_char() == '>' {
                    break;
                }
            }
        }
    }
    None
}

fn visit_attibutes(iter: &mut IntoIter) -> Attributes {
    let mut attributes: Attributes = HashMap::new();
    while let Some((name, value)) = visit_attibute(iter) {
        attributes.insert(name, value);
    }
    attributes
}
