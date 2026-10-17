use crate::error::{XmlError, XmlResult};
use crate::node::{Element, Node};

/// A parsed document: its top level nodes, in document order.
///
/// That is the XML declaration, any comments and processing instructions around
/// it, the document type declaration, and exactly one root element. Stray text
/// between them is kept as a sibling rather than rejected, which is the one
/// place this parser is looser than a validating one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    nodes: Vec<Node>,
}

impl Document {
    pub(crate) fn from_nodes(nodes: Vec<Node>) -> Self {
        Self { nodes }
    }

    /// Every top level node, in document order.
    #[must_use]
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// The root element.
    #[must_use]
    pub fn root(&self) -> Option<&Element> {
        self.nodes.iter().find_map(Node::as_element)
    }

    /// Fails unless the declaration asks for UTF-8, or says nothing at all.
    ///
    /// Only [`parse_bytes`](crate::parse_bytes) has to ask: an XML document
    /// without a declaration is UTF-8, and one that names another encoding is
    /// rejected rather than mis-decoded.
    pub(crate) fn check_encoding(&self) -> XmlResult<()> {
        let declaration = self.nodes.iter().find_map(|node| match node {
            Node::Declaration(declaration) => Some(declaration.as_str()),
            _ => None,
        });
        let Some(encoding) = declaration.and_then(declared_encoding) else {
            return Ok(());
        };
        if encoding.eq_ignore_ascii_case("utf-8") {
            Ok(())
        } else {
            Err(XmlError::UnsupportedEncoding(encoding.to_owned()))
        }
    }
}

/// Reads the `encoding` pseudo-attribute out of a declaration's contents.
///
/// The declaration is not markup, so this walks the text rather than reusing
/// the element attribute parser. A value that merely contains the word
/// `encoding` is skipped: only one followed by `=` and a quoted string counts.
fn declared_encoding(declaration: &str) -> Option<&str> {
    const KEY: &str = "encoding";

    let mut from = 0;
    while let Some(at) = declaration[from..].find(KEY) {
        let at = from + at;
        from = at + KEY.len();

        let Some(after) = declaration[from..].trim_start().strip_prefix('=') else {
            continue;
        };
        let value = after.trim_start();
        let quote = value.chars().next()?;
        if quote != '"' && quote != '\'' {
            return None;
        }
        let value = &value[quote.len_utf8()..];
        return value.find(quote).map(|end| &value[..end]);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_the_encoding_out_of_a_declaration() {
        let expected = Some("UTF-8");
        assert_eq!(
            declared_encoding("version=\"1.0\" encoding=\"UTF-8\""),
            expected
        );
        let expected = Some("iso-8859-1");
        assert_eq!(declared_encoding("encoding='iso-8859-1'"), expected);
    }

    #[test]
    fn a_declaration_without_an_encoding_reports_none() {
        let expected = None;
        assert_eq!(declared_encoding("version=\"1.0\""), expected);
    }

    #[test]
    fn the_word_encoding_inside_a_value_is_not_the_attribute() {
        let expected = None;
        assert_eq!(declared_encoding("standalone=\"encoding\""), expected);
    }

    #[test]
    fn a_document_with_no_declaration_accepts_any_encoding() {
        let document = Document::from_nodes(vec![Node::Element(Element::new("a"))]);
        assert!(document.check_encoding().is_ok());
    }

    #[test]
    fn a_document_declaring_another_encoding_is_rejected() {
        let document = Document::from_nodes(vec![
            Node::Declaration(String::from("version=\"1.0\" encoding=\"latin-1\"")),
            Node::Element(Element::new("a")),
        ]);
        let expected = "unsupported document encoding 'latin-1'";
        assert_eq!(document.check_encoding().unwrap_err().to_string(), expected);
    }
}
