use std::{collections::HashMap, process::Output, str::FromStr};

use proc_macro::{
    Delimiter, Ident, Literal, Punct, Spacing, TokenStream, TokenTree, token_stream::IntoIter,
};

pub fn parse_rsx(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();
    parse_rsx_stream(&mut iter)
}

pub fn parse_rsx_stream(iter: &mut IntoIter) -> TokenStream {
    let mut output = TokenStream::new();
    let mut is_rsx = true;
    let mut is_opening_tag = false;
    let mut is_inside_tag = false;
    let mut is_closing_tag = false;
    let mut is_attribute_name = false;
    let mut is_attribute_value = false;
    let mut is_children = false;
    let mut current_tag: Option<Ident> = None;
    let mut current_body: Option<TokenStream> = None;

    while let Some(token) = iter.next() {
        println!("token: {token:?}");
        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '<' {
                    is_opening_tag = true;
                }
                if is_opening_tag && punct.as_char() == '/' {
                    is_opening_tag = false;
                    is_closing_tag = true;
                }
                if punct.as_char() == '=' {
                    if is_attribute_name {
                        output.extend(vec![Punct::new(':', Spacing::Alone)]);
                        is_attribute_value = true;
                    }
                }
                if punct.as_char() == '>' {
                    if is_inside_tag {
                        is_children = true;
                        is_inside_tag = false;
                        current_body = Some(parse_rsx_stream(iter));
                    }
                    if is_closing_tag {
                        is_closing_tag = false;
                        let tag = current_tag.unwrap();
                        let body = current_body.unwrap();
                        let code = format!("{tag}::render({tag}::Props {{ {body}");
                        if let Ok(stream) = TokenStream::from_str(&code) {
                            output.extend(stream);
                        }
                    }
                }
            }
            TokenTree::Ident(ident) => {
                if is_opening_tag {
                    current_tag = Some(ident);
                    is_inside_tag = true;
                }
                if is_closing_tag {
                    iter.next();
                }
            }
            TokenTree::Group(group) => {
                if group.delimiter() == Delimiter::Brace {
                    if is_attribute_value {
                        output.extend(group.stream());
                        output.extend(vec![Punct::new(',', Spacing::Alone)]);
                    }
                    if is_children {
                        output.extend(vec![Literal::from_str("children").unwrap()]);
                        output.extend(vec![Punct::new(':', Spacing::Alone)]);
                        output.extend(group.stream());
                    }
                }
            }
            TokenTree::Literal(literal) => {
                if is_attribute_value {
                    output.extend(vec![literal]);
                    output.extend(vec![Punct::new(',', Spacing::Alone)]);
                }
            }
        }
    }
    println!("output: {output:?}");
    output
}

fn parse_tag(tag_name: Ident, iter: &mut IntoIter) -> TokenStream {
    let mut output = TokenStream::new();
    let mut is_attribute_value = false;

    let mut attributes: HashMap<Ident, TokenStream> = Vec::new();
    while let Some(token) = iter.next() {
        println!("token: {token:?}");
        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '=' {
                    output.extend(vec![Punct::new(':', Spacing::Alone)]);
                    is_attribute_value = true;
                }
            }
            TokenTree::Literal(literal) => {
                if is_attribute_value {
                    if let Some(attribute) = parse_attribute(iter) {
                        attributes.push(attribute);
                    }
                } else {
                    output.extend(vec![literal]);
                }
            }

            _ => {}
        }
    }
    output
}

fn parse_attribute(iter: &mut IntoIter) -> Option<TokenStream> {
    while let Some(token) = iter.next() {
        match token {
            TokenTree::Punct(punct) => {
                if punct.as_char() == '>' {
                    return None;
                }
            }
            TokenTree::Group(group) => {
                if group.delimiter() == Delimiter::Brace {
                    return Some(group.stream());
                }
            }
            TokenTree::Literal(literal) => {
                let mut stream = TokenStream::new();
                stream.extend(vec![ literal ]);
                return Some(stream)
            }
            _ => {}
        }
    }
    None
}
