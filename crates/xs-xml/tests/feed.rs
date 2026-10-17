//! End to end parses of documents shaped like the ones this workspace would
//! actually read: a feed, a sitemap, and a config file with a mixed content
//! root.

use xs_xml::{Node, XmlError, parse_bytes};

const FEED: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0" xmlns:media="http://search.yahoo.com/mrss/">
  <channel>
    <title>Hello &amp; welcome</title>
    <link>https://example.org</link>
    <item>
      <title>First &lt;post&gt;</title>
      <guid isPermaLink="false">42</guid>
      <enclosure url="https://example.org/a.mp4" length="1024" type="video/mp4"/>
      <description><![CDATA[<p>raw & unescaped</p>]]></description>
    </item>
    <item>
      <title>Second</title>
      <guid>43</guid>
    </item>
  </channel>
</rss>
"#;

#[test]
fn reads_a_feed() {
    let document = parse_bytes(FEED.as_bytes()).expect("the feed parses");
    let root = document.root().expect("the feed has a root");
    assert_eq!(root.name(), "rss");
    assert_eq!(root.attribute("version"), Some("2.0"));

    let channel = root.find("channel").expect("the channel is there");
    assert_eq!(
        channel.find("title").expect("title").text(),
        "Hello & welcome"
    );
    assert_eq!(
        channel.find("link").expect("link").text(),
        "https://example.org"
    );

    let items: Vec<String> = channel
        .children_named("item")
        .map(|item| item.find("title").expect("a title").text().into_owned())
        .collect();
    assert_eq!(items, ["First <post>", "Second"]);
}

#[test]
fn reads_the_attributes_of_an_empty_element() {
    let document = parse_bytes(FEED.as_bytes()).expect("the feed parses");
    let enclosure = document
        .root()
        .and_then(|root| root.find("channel/item/enclosure"))
        .expect("the enclosure is there");

    let expected = "https://example.org/a.mp4";
    assert_eq!(enclosure.attribute("url"), Some(expected));
    assert_eq!(enclosure.attribute("length"), Some("1024"));
    assert_eq!(enclosure.attribute("type"), Some("video/mp4"));
    assert_eq!(enclosure.attribute("missing"), None);
    assert_eq!(enclosure.children().len(), 0);
}

#[test]
fn a_cdata_section_is_a_child_of_its_own() {
    let document = parse_bytes(FEED.as_bytes()).expect("the feed parses");
    let description = document
        .root()
        .and_then(|root| root.find("channel/item/description"))
        .expect("the description is there");

    let expected = Node::CData(String::from("<p>raw & unescaped</p>"));
    assert_eq!(description.children(), [expected]);
    assert_eq!(description.text(), "<p>raw & unescaped</p>");
}

#[test]
fn whitespace_between_children_is_kept_as_text() {
    let document = parse_bytes(FEED.as_bytes()).expect("the feed parses");
    let channel = document
        .root()
        .and_then(|root| root.find("channel"))
        .expect("the channel is there");

    // One text child per gap between the elements, indentation included.
    let text: Vec<&str> = channel
        .children()
        .iter()
        .filter_map(Node::as_text)
        .collect();
    assert_eq!(text, ["\n    ", "\n    ", "\n    ", "\n    ", "\n  "]);
}

#[test]
fn reads_a_sitemap() {
    let sitemap = r#"<?xml version="1.0" encoding="UTF-8"?>
<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
  <url><loc>https://example.org/</loc><priority>1.0</priority></url>
  <url><loc>https://example.org/a</loc><lastmod>2026-10-17</lastmod></url>
</urlset>"#;

    let document = parse_bytes(sitemap.as_bytes()).expect("the sitemap parses");
    let root = document.root().expect("the sitemap has a root");
    let locations: Vec<String> = root
        .children_named("url")
        .map(|url| url.find("loc").expect("a loc").text().into_owned())
        .collect();
    assert_eq!(locations, ["https://example.org/", "https://example.org/a"]);

    let expected = "2026-10-17";
    let lastmod = root
        .children_named("url")
        .nth(1)
        .and_then(|url| url.child("lastmod"));
    assert_eq!(
        lastmod.map(|node| node.text().into_owned()).as_deref(),
        Some(expected)
    );
}

#[test]
fn reads_a_config_file_with_mixed_content() {
    let config = r#"<config>
  <!-- the port to listen on -->
  <server port="8080" host="0.0.0.0">
    <timeout>30</timeout>
    <timeout unit="s">60</timeout>
  </server>
</config>"#;

    let document = parse_bytes(config.as_bytes()).expect("the config parses");
    let server = document
        .root()
        .and_then(|root| root.find("server"))
        .expect("the server is there");
    assert_eq!(server.attribute("port"), Some("8080"));

    let timeouts: Vec<Option<&str>> = server
        .children_named("timeout")
        .map(|timeout| timeout.attribute("unit"))
        .collect();
    assert_eq!(timeouts, [None, Some("s")]);

    let comment = document
        .root()
        .expect("the config has a root")
        .children()
        .iter()
        .find(|node| matches!(node, Node::Comment(_)));
    assert_eq!(
        comment,
        Some(&Node::Comment(String::from(" the port to listen on ")))
    );
}

#[test]
fn a_document_that_is_not_xml_is_rejected() {
    let expected = "closing tag 'body' does not close 'p'";
    let error = parse_bytes(b"<html><body><p>oops</body></html>")
        .expect_err("unbalanced tags are rejected");
    assert!(matches!(error, XmlError::MismatchedTag { .. }));
    assert_eq!(error.to_string(), expected);
}
