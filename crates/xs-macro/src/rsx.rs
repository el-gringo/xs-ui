extern crate proc_macro;
use proc_macro::{Delimiter, Group, Ident, Punct, Spacing, TokenStream, TokenTree, token_stream::IntoIter};

type Attributes = Vec<(Ident, TokenStream)>;
#[derive(Debug)]
struct Node {
    name: Ident,
    attributes: Attributes,
    children: Vec<Node>,
}

impl Node {
    pub fn tokenize(&mut self) -> TokenStream {
        let mut output = TokenStream::new();
        let name = &self.name;
        // let children = self
        //     .children
        //     .iter_mut()
        //     .map(|node| node.tokenize().to_string())
        //     .collect::<Vec<String>>()
        //     .join(", ");
        // self.attributes
        //     .insert(
        //         "children".to_string(),
        //         TokenStream::from_str(
        //             format!("vec![{children}]").as_str()
        //         ).unwrap()
        //     );
        // let props = self
        //     .attributes
        //     .iter()
        //     .map(|(name, value)| format!("{name}: {value}"))
        //     .collect::<Vec<String>>()
        //     .join(", ");
        // let code = format!("{name}::render({name}::Props {{ {props} }})");
        println!("attributes: {:?}", self.attributes);
        let span = name.span();
        output.extend(vec![ Ident::new(&name.to_string(), span) ]);
        output.extend(vec![ Punct::new(':', Spacing::Joint), Punct::new(':', Spacing::Joint) ]);
        output.extend(vec![ Ident::new("render", span.end() ) ]);

        let mut render_body = TokenStream::new();
        let mut props_body = TokenStream::new();

        render_body.extend(vec![ Ident::new(&name.to_string(), span.end()) ]);
        render_body.extend(vec![ Punct::new(':', Spacing::Joint), Punct::new(':', Spacing::Joint) ]);
        render_body.extend(vec![ Ident::new("Props", span.end() ) ]);
        for attribute in self.attributes.clone() {
            let (name, value) = attribute;
            props_body.extend(vec![ name ]);
            props_body.extend(vec![ Punct::new(':', Spacing::Joint) ]);
            props_body.extend(value);
            props_body.extend(vec![ Punct::new(',', Spacing::Joint) ]);
        }
        props_body.extend(vec![ Ident::new("children", span ) ]);
        props_body.extend(vec![ Punct::new(':', Spacing::Joint) ]);

        props_body.extend(vec![ Ident::new("vec", span ) ]);
        props_body.extend(vec![ Punct::new('!', Spacing::Joint) ]);


        let mut children_body = TokenStream::new();
        for child in self.children.iter_mut() {
            children_body.extend(child.tokenize());
            children_body.extend(vec![ Punct::new(',', Spacing::Joint) ]);
        }
        props_body.extend(vec![ Group::new(Delimiter::Bracket, children_body) ]);
        render_body.extend(vec![ Group::new(Delimiter::Brace, props_body) ]);
        output.extend(vec![ Group::new(Delimiter::Parenthesis, render_body) ]);
        println!("output: {output}");
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
                    let attributes = visit_attibutes(iter);
                    let mut children: Vec<Node> = Vec::new();
                    while let Ok(child) = visit_tag(iter) {
                        children.push(child);
                    }
                    return Ok(Node {
                        name: ident,
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

fn visit_attibute(iter: &mut IntoIter) -> Option<(Ident, TokenStream)> {
    let mut name: Option<Ident> = None;
    for token in iter.by_ref() {
        match token {
            TokenTree::Ident(ident) => {
                name = Some(ident);
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
    let mut attributes: Attributes = Vec::new();
    while let Some((name, value)) = visit_attibute(iter) {
        attributes.push((name, value));
    }
    attributes
}
