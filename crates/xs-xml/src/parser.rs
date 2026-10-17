use std::io::Read;

use crate::document::Document;
use crate::error::{XmlError, XmlResult};
use crate::escape::unescape;
use crate::node::{Attribute, Element, Node};

/// How deep elements may be nested before the parser gives up.
///
/// The parser recurses, so this is what keeps a hostile or merely broken
/// document from taking the stack with it. 128 is far beyond anything hand
/// written markup reaches.
const MAX_DEPTH: usize = 128;

/// Parses a document from a string, which is assumed to be UTF-8 already.
///
/// No encoding is checked: a string has none, so a declaration naming something
/// else is not an error here. Use [`parse_bytes`] for bytes off disk or off the
/// network.
///
/// # Errors
///
/// [`XmlError`] for anything the grammar does not allow: an unbalanced tag, an
/// attribute with no value, an entity this crate cannot resolve, a document with
/// no element in it or with two.
pub fn parse(input: &str) -> XmlResult<Document> {
    Parser::new(input).parse()
}

/// Parses a document from bytes.
///
/// A UTF-8 byte order mark is skipped, and a declaration naming an encoding
/// other than UTF-8 is rejected rather than mis-decoded.
///
/// # Errors
///
/// As [`parse`], plus [`XmlError::Utf8`] for bytes that are not UTF-8 and
/// [`XmlError::UnsupportedEncoding`] for a declaration naming another encoding.
pub fn parse_bytes(bytes: &[u8]) -> XmlResult<Document> {
    let input = std::str::from_utf8(bytes).map_err(XmlError::Utf8)?;
    let document = Parser::new(input.strip_prefix('\u{feff}').unwrap_or(input)).parse()?;
    document.check_encoding()?;
    Ok(document)
}

/// Parses a document read to the end from `reader`, as [`parse_bytes`] does.
///
/// # Errors
///
/// As [`parse_bytes`], plus [`XmlError::Io`] when the read fails.
pub fn parse_reader<R: Read>(mut reader: R) -> XmlResult<Document> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    parse_bytes(&bytes)
}

/// A cursor over the document, positioned a character at a time.
struct Parser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    /// Reads the top level nodes until the input runs out.
    fn parse(mut self) -> XmlResult<Document> {
        let mut nodes = Vec::new();
        let mut roots = 0;

        loop {
            self.skip_whitespace();
            if self.peek().is_none() {
                break;
            }
            let node = self.parse_node(0)?;
            if node.as_element().is_some() {
                roots += 1;
                if roots > 1 {
                    return Err(XmlError::MultipleRoots);
                }
            }
            nodes.push(node);
        }

        if roots == 0 {
            return Err(XmlError::NoRoot);
        }
        Ok(Document::from_nodes(nodes))
    }

    /// One node at the cursor, which is at a `<` unless the caller has already
    /// established that text follows.
    fn parse_node(&mut self, depth: usize) -> XmlResult<Node> {
        if self.peek() != Some('<') {
            return self.parse_text();
        }
        if self.eat("<!--") {
            return Ok(Node::Comment(self.read_until("-->", "a comment")?));
        }
        if self.eat("<![CDATA[") {
            return Ok(Node::CData(self.read_until("]]>", "a CDATA section")?));
        }
        if self.eat("<?") {
            return self.parse_processing_instruction();
        }
        if self.eat("<!") {
            return self.parse_document_type();
        }
        if self.eat("</") {
            return Err(XmlError::Unexpected {
                expected: "an element".to_owned(),
                found: Some('/'),
            });
        }
        self.parse_element(depth)
    }

    fn parse_element(&mut self, depth: usize) -> XmlResult<Node> {
        if depth >= MAX_DEPTH {
            return Err(XmlError::TooDeep(MAX_DEPTH));
        }
        self.pos += 1; // '<'
        let name = self.parse_name()?;

        let mut attributes = Vec::new();
        let mut children = Vec::new();
        if !self.parse_attributes(&mut attributes)? {
            loop {
                if self.eat("</") {
                    let closing = self.parse_name()?;
                    self.skip_whitespace();
                    if closing != name {
                        return Err(XmlError::MismatchedTag {
                            expected: name,
                            found: closing,
                        });
                    }
                    if !self.eat(">") {
                        return Err(XmlError::Unterminated("a closing tag"));
                    }
                    break;
                }
                if self.peek().is_none() {
                    return Err(XmlError::UnexpectedEnd);
                }
                children.push(self.parse_node(depth + 1)?);
            }
        }

        Ok(Node::Element(Element::from_parts(
            name, attributes, children,
        )))
    }

    /// Reads attributes up to `>` or `/>`, reporting which of the two it found.
    fn parse_attributes(&mut self, attributes: &mut Vec<Attribute>) -> XmlResult<bool> {
        loop {
            self.skip_whitespace();
            match self.peek() {
                None => return Err(XmlError::UnexpectedEnd),
                Some('>') => {
                    self.pos += 1;
                    return Ok(false);
                }
                Some('/') => {
                    if self.eat("/>") {
                        return Ok(true);
                    }
                    return Err(XmlError::Unterminated("a self closing tag"));
                }
                Some(_) => {
                    let attribute = self.parse_attribute(attributes)?;
                    attributes.push(attribute);
                }
            }
        }
    }

    fn parse_attribute(&mut self, siblings: &[Attribute]) -> XmlResult<Attribute> {
        let name = self.parse_name()?;
        self.skip_whitespace();
        if !self.eat("=") {
            return Err(XmlError::Unexpected {
                expected: format!("'=' after '{name}'"),
                found: self.peek(),
            });
        }
        self.skip_whitespace();

        let Some(quote @ ('"' | '\'')) = self.peek() else {
            return Err(XmlError::UnquotedAttribute(name));
        };
        self.pos += quote.len_utf8();

        // XML turns the whitespace in an attribute value into spaces, so the
        // same value spread over several lines is the same string as one line.
        let input = self.input;
        let start = self.pos;
        let Some(end) = input[start..].find(quote) else {
            return Err(XmlError::Unterminated("an attribute value"));
        };
        let value = normalize(&input[start..start + end]);
        self.pos = start + end + quote.len_utf8();

        if siblings.iter().any(|sibling| sibling.name() == name) {
            return Err(XmlError::DuplicateAttribute(name));
        }
        Ok(Attribute::new(name, unescape(&value)?.into_owned()))
    }

    fn parse_text(&mut self) -> XmlResult<Node> {
        let input = self.input;
        let start = self.pos;
        while let Some(c) = self.peek() {
            if c == '<' {
                break;
            }
            self.pos += c.len_utf8();
        }
        let text = unescape(&input[start..self.pos])?;
        Ok(Node::Text(text.into_owned()))
    }

    fn parse_processing_instruction(&mut self) -> XmlResult<Node> {
        let target = self.parse_name()?;
        let data = self.read_until("?>", "a processing instruction")?;
        let data = data.trim().to_owned();
        if target == "xml" {
            Ok(Node::Declaration(data))
        } else {
            Ok(Node::ProcessingInstruction { target, data })
        }
    }

    /// Skips a `<!DOCTYPE .. >` declaration, internal subset and all, keeping
    /// its text. Nothing in it is interpreted: a DTD can declare entities and
    /// default attributes, and acting on those is a different parser.
    fn parse_document_type(&mut self) -> XmlResult<Node> {
        let start = self.pos;
        let mut subset = 0usize;
        let mut quote: Option<char> = None;

        loop {
            let Some(c) = self.bump() else {
                return Err(XmlError::Unterminated("a document type declaration"));
            };
            match quote {
                Some(open) if c == open => quote = None,
                Some(_) => {}
                None => match c {
                    '"' | '\'' => quote = Some(c),
                    '[' => subset += 1,
                    ']' => subset = subset.saturating_sub(1),
                    '>' if subset == 0 => break,
                    _ => {}
                },
            }
        }

        let input = self.input;
        Ok(Node::DocumentType(
            input[start..self.pos - 1].trim().to_owned(),
        ))
    }

    /// Reads up to `terminator`, leaving the cursor just past it.
    fn read_until(&mut self, terminator: &str, what: &'static str) -> XmlResult<String> {
        let input = self.input;
        let start = self.pos;
        let Some(end) = input[start..].find(terminator) else {
            return Err(XmlError::Unterminated(what));
        };
        self.pos = start + end + terminator.len();
        Ok(input[start..start + end].to_owned())
    }

    fn parse_name(&mut self) -> XmlResult<String> {
        let Some(c) = self.peek() else {
            return Err(XmlError::Unexpected {
                expected: "a name".to_owned(),
                found: None,
            });
        };
        if !is_name_start(c) {
            return Err(XmlError::InvalidName(c.to_string()));
        }

        let start = self.pos;
        self.pos += c.len_utf8();
        while let Some(c) = self.peek() {
            if !is_name_char(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
        Ok(self.input[start..self.pos].to_owned())
    }

    fn skip_whitespace(&mut self) {
        // XML's whitespace is exactly these four; `char::is_whitespace` would
        // also swallow the Unicode spaces, which are ordinary content.
        while matches!(self.peek(), Some(' ' | '\t' | '\n' | '\r')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += c.len_utf8();
        Some(c)
    }

    /// Consumes `expected` if it is what comes next, and says whether it was.
    fn eat(&mut self, expected: &str) -> bool {
        if self.input[self.pos..].starts_with(expected) {
            self.pos += expected.len();
            true
        } else {
            false
        }
    }
}

/// The XML whitespace replacement for an attribute value.
fn normalize(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}

/// Whether `c` can start a name.
///
/// The rules are the ASCII subset of the spec's plus every non-ASCII character:
/// an exact table of Unicode name characters is more than this crate needs, and
/// being permissive here costs nothing a caller can see, since a name nobody
/// recognises is simply a name nobody looks up.
fn is_name_start(c: char) -> bool {
    c.is_ascii_alphabetic() || c == '_' || c == ':' || !c.is_ascii()
}

/// Whether `c` can continue a name.
fn is_name_char(c: char) -> bool {
    is_name_start(c) || c.is_ascii_digit() || c == '-' || c == '.'
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root(input: &str) -> Element {
        parse(input)
            .expect("parses")
            .root()
            .expect("has a root")
            .clone()
    }

    #[test]
    fn reads_a_self_closing_element() {
        let element = root("<div/>");
        let expected = "div";
        assert_eq!(element.name(), expected);
        assert_eq!(element.children().len(), 0);
    }

    #[test]
    fn reads_attributes_in_both_quote_styles() {
        let element = root("<a href='x' class=\"row\"/>");
        assert_eq!(element.attribute("href"), Some("x"));
        assert_eq!(element.attribute("class"), Some("row"));
    }

    #[test]
    fn whitespace_inside_a_tag_is_ignored() {
        let element = root("<a\n  href = \"x\"\n/>");
        assert_eq!(element.attribute("href"), Some("x"));
    }

    #[test]
    fn text_and_nested_elements_keep_their_order() {
        let element = root("<p>one<b/>two</p>");
        let children: Vec<String> = element
            .children()
            .iter()
            .map(|child| child.text().into_owned())
            .collect();
        let expected = vec!["one", "", "two"];
        assert_eq!(children, expected);
    }

    #[test]
    fn text_around_children_is_kept_verbatim() {
        let element = root("<a> x <b/> </a>");
        let expected = " x  ";
        assert_eq!(element.text(), expected);
    }

    #[test]
    fn a_less_than_sign_needs_escaping() {
        let element = root("<a>x &lt; y</a>");
        let expected = "x < y";
        assert_eq!(element.text(), expected);
    }

    #[test]
    fn the_declaration_is_kept_apart_from_the_root() {
        let document = parse("<?xml version=\"1.0\" encoding=\"UTF-8\"?><a/>").expect("parses");
        let expected = Some(&Node::Declaration(String::from(
            "version=\"1.0\" encoding=\"UTF-8\"",
        )));
        assert_eq!(document.nodes().first(), expected);
        assert_eq!(document.nodes().len(), 2);
    }

    #[test]
    fn a_processing_instruction_keeps_its_target_and_data() {
        let document = parse("<?xml-stylesheet href=\"a.css\"?><a/>").expect("parses");
        let expected = Some(&Node::ProcessingInstruction {
            target: String::from("xml-stylesheet"),
            data: String::from("href=\"a.css\""),
        });
        assert_eq!(document.nodes().first(), expected);
    }

    #[test]
    fn comments_and_cdata_are_kept_apart() {
        let element = root("<a><!-- <b/> --><![CDATA[<b/> & more]]></a>");
        let expected = vec![
            Node::Comment(String::from(" <b/> ")),
            Node::CData(String::from("<b/> & more")),
        ];
        assert_eq!(element.children(), expected);
    }

    #[test]
    fn cdata_contents_are_not_entity_resolved() {
        let element = root("<a><![CDATA[&amp;]]></a>");
        let expected = "&amp;";
        assert_eq!(element.text(), expected);
    }

    #[test]
    fn comments_keep_entities_literal() {
        let element = root("<a><!-- &amp; --></a>");
        let expected = Some(&Node::Comment(String::from(" &amp; ")));
        assert_eq!(element.children().first(), expected);
    }

    #[test]
    fn a_document_type_declaration_is_skipped_not_interpreted() {
        let document = parse("<!DOCTYPE a [<!ENTITY x \"y\">]><a/>").expect("parses");
        let expected = Some(&Node::DocumentType(String::from(
            "DOCTYPE a [<!ENTITY x \"y\">]",
        )));
        assert_eq!(document.nodes().first(), expected);
        assert_eq!(document.root().map(Element::name), Some("a"));
    }

    #[test]
    fn a_colon_in_a_name_is_left_to_the_caller() {
        let element = root("<svg:rect xlink:href=\"#a\"/>");
        assert_eq!(element.name(), "svg:rect");
        assert_eq!(element.attribute("xlink:href"), Some("#a"));
    }

    #[test]
    fn a_non_ascii_name_is_accepted() {
        let element = root("<café>x</café>");
        let expected = "café";
        assert_eq!(element.name(), expected);
    }

    #[test]
    fn whitespace_in_an_attribute_value_is_normalised() {
        let element = root("<a href=\"one\ntwo\tx\"/>");
        let expected = "one two x";
        assert_eq!(element.attribute("href"), Some(expected));
    }

    #[test]
    fn a_byte_order_mark_is_skipped() {
        let document = parse_bytes("\u{feff}<a/>".as_bytes()).expect("parses");
        let expected = "a";
        assert_eq!(document.root().map(Element::name), Some(expected));
    }

    #[test]
    fn bytes_that_are_not_utf8_are_rejected() {
        let error = parse_bytes(&[0xFF, 0xFE, b'<']).unwrap_err();
        assert!(std::error::Error::source(&error).is_some());
    }

    #[test]
    fn a_declared_encoding_other_than_utf8_is_rejected() {
        let input = "<?xml version=\"1.0\" encoding=\"ISO-8859-1\"?><a/>";
        let expected = "unsupported document encoding 'ISO-8859-1'";
        assert_eq!(
            parse_bytes(input.as_bytes()).unwrap_err().to_string(),
            expected
        );
    }

    #[test]
    fn a_reader_is_read_to_the_end() {
        let document = parse_reader("<a>x</a>".as_bytes()).expect("parses");
        let root = document.root().expect("has a root");
        assert_eq!(root.text(), "x");
    }

    #[test]
    fn a_closing_tag_must_match_the_element_it_closes() {
        let expected = "closing tag 'b' does not close 'a'";
        assert_eq!(parse("<a></b>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn an_unclosed_element_is_an_error() {
        let expected = "the input ends inside a construct";
        assert_eq!(parse("<a><b/>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn a_closing_tag_never_closed_is_an_error() {
        let expected = "unterminated a closing tag";
        assert_eq!(parse("<a></a x").unwrap_err().to_string(), expected);
    }

    #[test]
    fn an_unterminated_comment_is_an_error() {
        let expected = "unterminated a comment";
        assert_eq!(parse("<a><!-- oops</a>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn an_unterminated_cdata_section_is_an_error() {
        let expected = "unterminated a CDATA section";
        assert_eq!(
            parse("<a><![CDATA[ oops</a>").unwrap_err().to_string(),
            expected
        );
    }

    #[test]
    fn a_tag_with_no_name_is_an_error() {
        let expected = "invalid name '>'";
        assert_eq!(parse("<>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn a_stray_closing_tag_is_an_error() {
        let expected = "expected an element, found '/'";
        assert_eq!(parse("<a></a></b>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn an_unquoted_attribute_value_is_an_error() {
        let expected = "the value of 'href' is not quoted";
        assert_eq!(parse("<a href=x/>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn a_duplicated_attribute_is_an_error() {
        let expected = "attribute 'href' appears twice";
        assert_eq!(
            parse("<a href=\"x\" href=\"y\"/>").unwrap_err().to_string(),
            expected
        );
    }

    #[test]
    fn an_attribute_with_no_equals_is_an_error() {
        let expected = "expected '=' after 'href', found '/'";
        assert_eq!(parse("<a href/>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn a_second_root_element_is_an_error() {
        let expected = "the document has more than one root element";
        assert_eq!(parse("<a/><b/>").unwrap_err().to_string(), expected);
    }

    #[test]
    fn a_document_with_no_element_is_an_error() {
        let expected = "the document has no root element";
        assert_eq!(
            parse("<!-- nothing here -->").unwrap_err().to_string(),
            expected
        );
    }

    #[test]
    fn empty_input_is_an_error() {
        let expected = "the document has no root element";
        assert_eq!(parse("   \n ").unwrap_err().to_string(), expected);
    }

    #[test]
    fn nesting_deeper_than_the_limit_is_an_error() {
        let deep = format!(
            "{}{}",
            "<a>".repeat(MAX_DEPTH + 1),
            "</a>".repeat(MAX_DEPTH + 1)
        );
        let expected = format!("elements nested more than {MAX_DEPTH} deep");
        assert_eq!(parse(&deep).unwrap_err().to_string(), expected);
    }

    #[test]
    fn nesting_up_to_the_limit_is_accepted() {
        let deep = format!("{}{}", "<a>".repeat(MAX_DEPTH), "</a>".repeat(MAX_DEPTH));
        assert!(parse(&deep).is_ok());
    }
}
