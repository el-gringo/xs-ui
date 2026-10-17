//! A small XML parser: string or bytes in, a tree of [`Element`]s and
//! [`Node`]s out.
//!
//! The scope is deliberately narrow, because this workspace only reads a few
//! small documents and a parser that does everything is a parser nobody
//! understands:
//!
//! * **No DTD.** A `<!DOCTYPE>` is skipped, internal subset and all, and kept as
//!   [`Node::DocumentType`]. Entities and default attributes declared in one are
//!   not applied, so an entity this crate cannot resolve is an
//!   [`XmlError::UnknownEntity`] rather than a silent pass-through.
//! * **No namespaces.** A name is a name: `svg:rect` stays `svg:rect` and
//!   `xlink:href` stays `xlink:href`. Prefixing a lookup is the caller's job.
//! * **UTF-8 only.** [`parse_bytes`] skips a byte order mark and rejects a
//!   declaration naming any other encoding rather than mis-decoding it.
//! * **Not streaming, not writing.** Nothing here emits XML again.
//! * **Not validating.** A stray text node between the root and the rest is
//!   kept rather than rejected, and only the structural mistakes a parser can
//!   catch - an unbalanced tag, a duplicate attribute - are errors.
//!
//! # Examples
//!
//! ```
//! use xs_xml::{parse, Node};
//!
//! let document = parse(
//!     r#"<?xml version="1.0"?>
//!        <rss><channel><title>Hello &amp; welcome</title></channel></rss>"#,
//! )
//! .unwrap();
//!
//! let title = document.root().unwrap().find("channel/title").unwrap();
//! assert_eq!(title.text(), "Hello & welcome");
//!
//! // Comments, CDATA sections and processing instructions are kept, not dropped.
//! let document = parse("<a><!-- note --><![CDATA[<raw>]]></a>").unwrap();
//! let root = document.root().unwrap();
//! assert_eq!(root.children(), [
//!     Node::Comment(" note ".to_owned()),
//!     Node::CData("<raw>".to_owned()),
//! ]);
//! ```

#![warn(clippy::all, clippy::pedantic)]

mod document;
mod error;
mod escape;
mod node;
mod parser;

pub use document::Document;
pub use error::{XmlError, XmlResult};
pub use escape::{escape, unescape};
pub use node::{Attribute, Element, Node};
pub use parser::{parse, parse_bytes, parse_reader};
