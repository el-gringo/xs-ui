use std::borrow::Cow;

/// One node of a parsed document.
///
/// The variants cover everything the grammar can produce, including the ones
/// carry no data this crate acts on: a [`DocumentType`](Self::DocumentType) and
/// the XML declaration are kept verbatim rather than interpreted.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// An element, with its attributes and its children.
    Element(Element),
    /// Character data between tags, with entities already resolved.
    Text(String),
    /// A `<![CDATA[ .. ]]>` section, whose contents are **not** entity
    /// resolved: that is the whole point of the construct.
    CData(String),
    /// A `<!-- .. -->` comment.
    Comment(String),
    /// An `<?xml .. ?>` declaration, holding everything after `<?xml`.
    Declaration(String),
    /// Any other `<?target .. ?>` processing instruction.
    ProcessingInstruction {
        /// The instruction's target, `xml-stylesheet` and the like.
        target: String,
        /// Everything after the target, trimmed.
        data: String,
    },
    /// A `<!DOCTYPE .. >` declaration, holding everything after `<!`,
    /// including the internal subset. It is skipped, never interpreted.
    DocumentType(String),
}

impl Node {
    /// The element this node is, if it is one.
    #[must_use]
    pub fn as_element(&self) -> Option<&Element> {
        match self {
            Self::Element(element) => Some(element),
            _ => None,
        }
    }

    /// The character data this node holds, if it is text. A CDATA section is
    /// *not* text: it is reported by [`Node::as_cdata`] instead.
    #[must_use]
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(text) => Some(text),
            _ => None,
        }
    }

    /// The contents of a CDATA section, if this node is one.
    #[must_use]
    pub fn as_cdata(&self) -> Option<&str> {
        match self {
            Self::CData(text) => Some(text),
            _ => None,
        }
    }

    /// Every character below this node, concatenated, entities resolved.
    ///
    /// A CDATA section counts, since its characters are the document's
    /// characters; a comment does not.
    #[must_use]
    pub fn text(&self) -> Cow<'_, str> {
        match self {
            Self::Text(text) | Self::CData(text) => Cow::Borrowed(text),
            Self::Element(element) => element.text(),
            _ => Cow::Borrowed(""),
        }
    }
}

/// An element: a name, attributes, and children.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Element {
    name: String,
    attributes: Vec<Attribute>,
    children: Vec<Node>,
}

impl Element {
    /// An empty element with no attributes.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            attributes: Vec::new(),
            children: Vec::new(),
        }
    }

    /// An element carrying one more attribute.
    #[must_use]
    pub fn with_attribute(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.push(Attribute::new(name, value));
        self
    }

    /// An element carrying one more text child.
    #[must_use]
    pub fn with_text(mut self, text: impl Into<String>) -> Self {
        self.children.push(Node::Text(text.into()));
        self
    }

    /// An element carrying one more child of any kind.
    #[must_use]
    pub fn with_child(mut self, child: impl Into<Node>) -> Self {
        self.children.push(child.into());
        self
    }

    pub(crate) fn from_parts(
        name: String,
        attributes: Vec<Attribute>,
        children: Vec<Node>,
    ) -> Self {
        Self {
            name,
            attributes,
            children,
        }
    }

    /// The tag name, as written - a prefix is part of it, since namespaces are
    /// not resolved here.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Every attribute, in document order.
    #[must_use]
    pub fn attributes(&self) -> &[Attribute] {
        &self.attributes
    }

    /// The value of one attribute, with entities already resolved.
    #[must_use]
    pub fn attribute(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|attribute| attribute.name == name)
            .map(|attribute| attribute.value.as_str())
    }

    /// Every child, text and element alike, in document order.
    #[must_use]
    pub fn children(&self) -> &[Node] {
        &self.children
    }

    /// Every child element, skipping text and comments.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(Node::as_element)
    }

    /// The first child element with this name.
    #[must_use]
    pub fn child(&self, name: &str) -> Option<&Element> {
        self.elements().find(|element| element.name == name)
    }

    /// Every child element with this name.
    pub fn children_named<'a>(&'a self, name: &'a str) -> impl Iterator<Item = &'a Element> {
        self.elements().filter(move |element| element.name == name)
    }

    /// The first element at this path, each step separated by `/`.
    ///
    /// An empty path is the element itself, and empty segments - which is what
    /// a leading or trailing `/` produces - are skipped, so `div/span` and
    /// `/div/span/` are the same path.
    #[must_use]
    pub fn find(&self, path: &str) -> Option<&Element> {
        let mut current = self;
        for segment in path.split('/') {
            if segment.is_empty() {
                continue;
            }
            current = current.child(segment)?;
        }
        Some(current)
    }

    /// Every character below this element, concatenated, entities resolved.
    ///
    /// Text is whitespace significant and nothing is trimmed: the children of
    /// `<a> x <b/> </a>` give `" x  "`, so a caller that wants the shape of the
    /// markup rather than its content wants [`Element::children`].
    #[must_use]
    pub fn text(&self) -> Cow<'_, str> {
        // Only the one text child can be borrowed; anything else is built.
        if let [Node::Text(text) | Node::CData(text)] = self.children.as_slice() {
            return Cow::Borrowed(text);
        }
        let mut text = String::new();
        for child in &self.children {
            text.push_str(&child.text());
        }
        Cow::Owned(text)
    }
}

impl From<Element> for Node {
    fn from(element: Element) -> Self {
        Self::Element(element)
    }
}

impl From<&str> for Node {
    fn from(text: &str) -> Self {
        Self::Text(text.to_owned())
    }
}

impl From<String> for Node {
    fn from(text: String) -> Self {
        Self::Text(text)
    }
}

/// One attribute of an [`Element`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attribute {
    name: String,
    value: String,
}

impl Attribute {
    /// An attribute with no value of its own, as parsed from `name="value"`.
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }

    /// The attribute name, as written.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The attribute value, entities resolved and whitespace normalised.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element() -> Element {
        let child = Element::new("span").with_text("inner");
        Element::new("div")
            .with_attribute("class", "row")
            .with_text("before")
            .with_attribute("id", "top")
            .with_child(child)
            .with_text("after")
    }

    #[test]
    fn attributes_keep_document_order_and_resolve_by_name() {
        let element = element();
        let expected = vec!["class", "id"];
        let names: Vec<&str> = element.attributes().iter().map(Attribute::name).collect();
        assert_eq!(names, expected);
        assert_eq!(element.attribute("id"), Some("top"));
        assert_eq!(element.attribute("missing"), None);
    }

    #[test]
    fn elements_skips_text_children() {
        let element = element();
        let expected = 1;
        let found = element.elements().count();
        assert_eq!(found, expected);
    }

    #[test]
    fn text_concatenates_every_child_in_order() {
        let element = element();
        let expected = "beforeinnerafter";
        assert_eq!(element.text(), expected);
    }

    #[test]
    fn text_borrows_the_only_piece_it_has() {
        let element = Element::new("div").with_text("only");
        let expected = Cow::Borrowed("only");
        assert!(matches!(element.text(), Cow::Borrowed(_)));
        assert_eq!(element.text(), expected);
    }

    #[test]
    fn an_element_with_no_text_has_empty_text() {
        let element = Element::new("div").with_attribute("a", "b");
        let expected = "";
        assert_eq!(element.text(), expected);
    }

    #[test]
    fn find_walks_a_slash_separated_path() {
        let root = Element::new("html")
            .with_child(Element::new("body").with_child(Element::new("div").with_text("x")));
        let found = root.find("body/div").expect("body/div is there");
        assert_eq!(found.name(), "div");
        assert_eq!(found.text(), "x");
    }

    #[test]
    fn find_skips_empty_segments() {
        let root = Element::new("html").with_child(Element::new("body"));
        let expected = "body";
        assert_eq!(root.find("/body/").map(Element::name), Some(expected));
    }

    #[test]
    fn find_returns_nothing_for_a_path_that_stops_early() {
        let root = Element::new("html").with_child(Element::new("body"));
        let expected = None;
        assert_eq!(root.find("body/span").map(Element::name), expected);
    }

    #[test]
    fn node_text_reports_each_own_kind() {
        assert_eq!(Node::Text(String::from("x")).text(), "x");
        assert_eq!(Node::CData(String::from("x")).text(), "x");
        assert_eq!(Node::Comment(String::from("x")).text(), "");
        assert_eq!(
            Node::Comment(String::from("x")).as_text().map(str::len),
            None
        );
    }
}
