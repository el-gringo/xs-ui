use std::path::{Path, PathBuf};

/// Reads the html file the dev server serves and injects the wasm loader into
/// it. The file on disk stays free of any xs-ui specific markup, the tool owns
/// what has to run in the page.
pub struct Html {
    /// The file as it is on disk, injected into on every request so editing it
    /// is picked up without a restart.
    source: String,
    /// Directory the html lives in, the most specific place to look for its
    /// assets.
    directory: PathBuf,
}

impl Html {
    pub fn read(path: &Path) -> Result<Html, String> {
        let source = std::fs::read_to_string(path)
            .map_err(|error| format!("unable to read {}: {error}", path.display()))?;
        Ok(Html {
            source,
            directory: directory_of(path),
        })
    }

    /// Every directory an asset is looked up in, most specific first.
    ///
    /// The page's own directory, then the one the binding system ships from.
    /// The fallback is what makes the html argument mean *which page* rather
    /// than *where the assets are*: every xs-ui page refers to `assets/js/boot.js`,
    /// and there is exactly one such tree in a project - the one next to
    /// `crates/xs-js-sys/html/index.html`. A page kept somewhere else, a
    /// scratch one in the project root while debugging, says nothing about
    /// where its assets are, so asking for it 404s unless the shared tree is
    /// still in the list.
    ///
    /// The page's own `assets` wins where the two overlap, which is what lets a
    /// project override the shipped bindings rather than edit them.
    pub fn asset_directories(&self) -> Vec<PathBuf> {
        let mut directories = vec![self.directory.clone()];
        if let Some(shared) = shared_directory()
            && shared != self.directory
        {
            directories.push(shared);
        }
        directories
    }

    /// The page with the wasm loader injected. It goes where the
    /// `<link rel="rust" />` marker is, and at the end of the body when the page
    /// has no marker.
    ///
    /// `wasm_url` is what the page fetches, `dev` adds the hot reload client.
    pub fn render(&self, wasm_url: &str, dev: bool) -> String {
        let script = loader(wasm_url, dev);
        inject_at(&self.source, &script).unwrap_or_else(|| inject(&self.source, &script))
    }
}

/// Where the shipped page keeps its assets, when `xs` is run inside the xs-ui
/// repository itself.
///
/// Derived from the shared page's path rather than spelled out again, so there
/// is one place that knows where the binding system lives. Outside this
/// repository the path does not resolve and the list is just the page's own
/// directory, which is where a project that ships its own assets puts them.
fn shared_directory() -> Option<PathBuf> {
    Path::new(crate::options::SHARED_HTML)
        .parent()
        .filter(|parent| parent.is_dir())
        .map(Path::to_path_buf)
}

/// The directory a page keeps its assets in.
///
/// `parent` of a bare filename is an empty path rather than `.`, which `join`
/// happens to tolerate but which reads like a mistake everywhere it is
/// printed.
fn directory_of(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// The script tags to append to the body.
pub fn loader(wasm_url: &str, dev: bool) -> String {
    let mut script = String::new();
    script.push_str("<!-- injected by xs -->\n");
    script.push_str(r#"<script src="assets/js/boot.js"></script>"#);
    script.push('\n');
    script.push_str("<script>\n");
    script.push_str(&format!("  loadWasm({:?});\n", wasm_url));

    if dev {
        // A rebuild reloads the page rather than instantiating the new wasm
        // next to the running one: an app owns a canvas and a frame loop, so
        // two instances in one page would fight over the same context.
        script.push_str(
            r#"  (function () {
    var protocol = location.protocol === "https:" ? "wss" : "ws";
    var ws = new WebSocket(protocol + "://" + location.host + "/ws");
    ws.onmessage = function (event) {
      if (/\.wasm(\?.*)?$/.test(event.data)) {
        location.reload();
      }
    };
    window.ws = ws;
  })();
"#,
        );
    }

    script.push_str("</script>\n");
    script
}

/// Byte range of the last `<link rel="rust" />`, the tag the loader replaces.
pub fn find_rust_link(html: &str) -> Option<(usize, usize)> {
    let lower = html.to_lowercase();
    let mut from = 0;
    let mut found = None;
    while let Some(at) = lower[from..].find("<link") {
        let at = from + at;
        let end = lower[at..].find('>').map(|close| at + close + 1)?;
        // `<link ... >`, the closing slash of a self closing tag included.
        let attributes = &html[at + "<link".len()..end];
        if has_rel_rust(attributes) {
            found = Some((at, end));
        }
        from = end;
    }
    found
}

/// True when the tag carries `rel="rust"`, in any attribute order.
fn has_rel_rust(attributes: &str) -> bool {
    let lower = attributes.to_lowercase();
    let mut from = 0;
    while let Some(at) = lower[from..].find("rel") {
        let at = from + at;
        let rest = lower[at + 3..].trim_start();
        let rest = match rest.strip_prefix('=') {
            Some(rest) => rest.trim_start(),
            None => {
                from = at + 3;
                continue;
            }
        };
        let value: String = match rest.chars().next() {
            Some(quote @ ('"' | '\'')) => rest[1..].chars().take_while(|c| *c != quote).collect(),
            _ => rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '/')
                .collect(),
        };
        if value.split_whitespace().any(|token| token == "rust") {
            return true;
        }
        from = at + 3;
    }
    false
}

/// Replaces the `<link rel="rust" />` marker with `markup`, `None` when the page
/// has no marker.
pub fn inject_at(html: &str, markup: &str) -> Option<String> {
    let (start, end) = find_rust_link(html)?;
    let mut out = String::with_capacity(html.len() + markup.len());
    out.push_str(&html[..start]);
    out.push_str(markup);
    out.push_str(&html[end..]);
    Some(out)
}

/// Inserts `markup` at the end of the body, falling back to the end of the
/// html and then to the end of the file when neither closing tag is there.
pub fn inject(html: &str, markup: &str) -> String {
    for closing in ["</body>", "</html>"] {
        if let Some(at) = find_last_tag(html, closing) {
            let mut out = String::with_capacity(html.len() + markup.len());
            out.push_str(&html[..at]);
            out.push_str(markup);
            out.push_str(&html[at..]);
            return out;
        }
    }
    let mut out = String::with_capacity(html.len() + markup.len() + 1);
    out.push_str(html);
    out.push('\n');
    out.push_str(markup);
    out
}

/// Case insensitive index of the last `tag`, ignoring what is inside comments so
/// a commented out `</body>` is not picked up.
fn find_last_tag(html: &str, tag: &str) -> Option<usize> {
    let lower = html.to_lowercase();
    let mut found = None;
    let mut search_from = 0;
    while let Some(at) = lower[search_from..].find(tag) {
        let at = search_from + at;
        let in_comment = lower[..at]
            .rfind("<!--")
            .is_some_and(|open| lower[open..at].find("-->").is_none());
        if !in_comment {
            found = Some(at);
        }
        search_from = at + tag.len();
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injects_before_the_closing_body() {
        let out = inject("<html><body><p>hi</p></body></html>", "<script>x</script>");
        assert_eq!(out, "<html><body><p>hi</p><script>x</script></body></html>");
    }

    #[test]
    fn is_case_insensitive() {
        let out = inject("<BODY>hi</BODY>", "x");
        assert_eq!(out, "<BODY>hix</BODY>");
    }

    #[test]
    fn falls_back_to_the_closing_html_tag() {
        let out = inject("<p>hi</p></html>", "x");
        assert_eq!(out, "<p>hi</p>x</html>");
    }

    #[test]
    fn appends_when_there_is_no_body() {
        assert_eq!(inject("<p>hi</p>", "x"), "<p>hi</p>\nx");
    }

    #[test]
    fn ignores_a_commented_out_body_tag() {
        let out = inject("<!-- </body> --><body>hi</body>", "x");
        assert_eq!(out, "<!-- </body> --><body>hix</body>");
    }

    #[test]
    fn replaces_the_rust_link() {
        let out = inject_at(
            "<body><link rel=\"rust\" /><p>hi</p></body>",
            "<script>x</script>",
        )
        .unwrap();
        assert_eq!(out, "<body><script>x</script><p>hi</p></body>");
    }

    #[test]
    fn the_rust_link_is_found_whatever_its_attributes() {
        // The rel matters, only the rust link is the marker.
        let out = inject_at(
            "<link data-x=\"1\" rel=\"rust\" /><link rel=\"manifest\" />",
            "x",
        )
        .unwrap();
        assert_eq!(out, "x<link rel=\"manifest\" />");
    }

    #[test]
    fn only_the_last_rust_link_is_replaced() {
        let out = inject_at("<link rel=\"rust\" /><p>a</p><link rel=\"rust\" />", "x").unwrap();
        assert_eq!(out, "<link rel=\"rust\" /><p>a</p>x");
    }

    #[test]
    fn falls_back_to_the_body_without_a_marker() {
        assert_eq!(find_rust_link("<p>hi</p>"), None);
    }

    #[test]
    fn dev_loader_has_the_websocket_client() {
        let script = loader("app.wasm", true);
        assert!(script.contains(r#"loadWasm("app.wasm")"#));
        assert!(script.contains("/ws"));
        assert!(script.contains("boot.js"));
        assert!(script.contains("location.reload()"));
    }

    #[test]
    fn release_loader_has_no_websocket_client() {
        let script = loader("app.wasm", false);
        assert!(script.contains(r#"loadWasm("app.wasm")"#));
        assert!(!script.contains("/ws"));
    }

    #[test]
    fn a_bare_filename_means_the_current_directory() {
        // `parent` of a bare filename is an empty path rather than `.`, and an
        // empty path is the kind of thing that reads as a mistake wherever it
        // is printed.
        assert_eq!(directory_of(Path::new("debug.html")), PathBuf::from("."));
        assert_eq!(
            directory_of(Path::new("pages/a.html")),
            PathBuf::from("pages")
        );
    }

    #[test]
    fn the_page_s_directory_is_searched_before_the_shared_one() {
        // Paths here are anchored on the repository rather than left relative,
        // because a test binary runs with the package directory as its working
        // directory and `../../crates/...` in an assertion is only meaningful on
        // the machine that wrote it.
        let Some(shared) = shared_directory().map(|d| canonical(&d)) else {
            return;
        };
        let root = shared
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .expect("crates/xs-js-sys/html is three directories down")
            .to_path_buf();
        let html = Html::read(&root.join("debug.html")).unwrap();
        assert_eq!(html.asset_directories(), vec![root, shared]);
    }

    #[test]
    fn the_shipped_page_is_not_also_its_own_fallback() {
        // The page's directory and the shared one are the same directory here,
        // and listing it twice would have every asset looked up twice.
        let Ok(html) = Html::read(&canonical(Path::new(crate::options::SHARED_HTML))) else {
            return;
        };
        let directories = html.asset_directories();
        assert_eq!(directories.len(), 1, "{directories:?}");
    }

    fn canonical(path: &Path) -> PathBuf {
        std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
}
