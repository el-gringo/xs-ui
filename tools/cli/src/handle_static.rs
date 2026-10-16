use std::io::Read;
use std::path::{Path, PathBuf};
use xs_http::{HttpHeaders, HttpRequest, HttpResponse};

use crate::html::Html;

/// Everything the server knows about the app it serves.
pub struct Site {
    /// The html file, read again on every request so editing it needs no
    /// restart.
    pub html: PathBuf,
    /// Where an asset is looked up, most specific first. Resolved once when the
    /// server starts, unlike the html itself: a page's *contents* change while
    /// you edit it, but where it lives cannot change under a running server.
    pub assets: Vec<PathBuf>,
    /// Name the wasm is served under.
    pub wasm_name: String,
    /// Where the wasm is on disk.
    pub wasm_path: PathBuf,
    /// Inject the hot reload client.
    pub dev: bool,
}

impl Site {
    /// Url the page and the hot reload use, cache busted so a rebuild is never
    /// served from the browser cache.
    pub fn wasm_url(&self, version: u32) -> String {
        format!("{}?v={version}", self.wasm_name)
    }

    /// The page with the wasm loader injected.
    pub fn render(&self, wasm_url: &str) -> Result<String, String> {
        Ok(Html::read(&self.html)?.render(wasm_url, self.dev))
    }
}

fn get_content_type(path: &Path) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };
    match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "ico" => "image/x-icon",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "wasm" => "application/wasm",
        "htm" | "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        "js" | "mjs" => "text/javascript; charset=utf8",
        "json" => "application/json",
        "css" => "text/css; charset=utf8",
        "webmanifest" => "application/manifest+json",
        _ => "text/plain; charset=utf8",
    }
}

fn server_error(error: &str) -> HttpResponse {
    let mut headers = HttpHeaders::new();
    headers.insert(
        "Content-Type".to_string(),
        "text/plain; charset=utf8".to_string(),
    );
    HttpResponse::new(500, headers, error.into())
}

fn not_found() -> HttpResponse {
    let mut headers = HttpHeaders::new();
    headers.insert(
        "Content-Type".to_string(),
        "text/plain; charset=utf8".to_string(),
    );
    HttpResponse::new(404, headers, "Not found".into())
}

fn respond(status: u16, content_type: &str, body: Vec<u8>) -> HttpResponse {
    let mut headers = HttpHeaders::new();
    headers.insert("Content-Type".to_string(), content_type.to_string());
    headers.insert("Content-Length".to_string(), body.len().to_string());
    // Nothing may be cached: the page is rebuilt while the browser is open.
    headers.insert("Cache-Control".to_string(), "no-store".to_string());
    HttpResponse::new(status, headers, body)
}

fn read_file(path: &Path) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut body = Vec::new();
    file.read_to_end(&mut body).ok()?;
    Some(body)
}

/// Serves the page with the wasm loader injected, the wasm itself, and the
/// assets the html refers to.
pub fn handle_static(request: HttpRequest, site: &Site, version: u32) {
    let pathname = match request.pathname.split_once('?') {
        Some((path, _)) => &path[1..],
        None => &request.pathname[1..],
    };

    let response = if is_page(&site.html, pathname) {
        // Both `/` and the page's own file name are the page. Serving the file
        // as a plain asset instead would hand the browser the html as it sits on
        // disk, which carries no loader and therefore never boots anything - a
        // server that looks broken rather than one that is.
        match site.render(&site.wasm_url(version)) {
            Ok(page) => respond(200, "text/html; charset=utf8", page.into_bytes()),
            Err(error) => server_error(&error),
        }
    } else if pathname == site.wasm_name {
        // The query is a cache buster, `app.wasm?v=3` is the same file.
        match read_file(&site.wasm_path) {
            Some(body) => respond(200, "application/wasm", body),
            None => not_found(),
        }
    } else {
        match resolve(&site.assets, pathname) {
            Some(path) => match read_file(&path) {
                Some(body) => respond(200, get_content_type(&path), body),
                None => not_found(),
            },
            None => not_found(),
        }
    };

    request.respond(response);
}

/// Whether `pathname` is the page itself: the root, or the file name the html
/// argument was given.
///
/// Matched on the last segment only, because the page does not have to be at the
/// root of what it is served from - `xs serve counter pages/debug.html` and a
/// link to `debug.html` both have to reach it.
fn is_page(html: &Path, pathname: &str) -> bool {
    if pathname.is_empty() {
        return true;
    }
    html.file_name()
        .is_some_and(|name| name == pathname.rsplit('/').next().unwrap_or(pathname))
}

/// Looks a request up in the directories an asset can live in, in order, so
/// `assets/js/boot.js` resolves the way the page asks for it even when the page
/// itself sits somewhere else.
fn resolve(directories: &[PathBuf], pathname: &str) -> Option<PathBuf> {
    if pathname.split('/').any(|segment| segment == "..") {
        return None;
    }
    directories
        .iter()
        .map(|directory| directory.join(pathname))
        .find(|path| path.is_file())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A throwaway tree shaped like a project: a page in one directory with no
    /// assets of its own, and the `assets` tree it would have to borrow.
    ///
    /// Built under the temp directory rather than pointed at the real one,
    /// because a test binary runs with the package directory as its working
    /// directory and `../../crates/...` in an assertion is only meaningful on
    /// the machine that wrote it.
    struct Fixture {
        root: PathBuf,
    }

    impl Fixture {
        fn new(name: &str) -> Fixture {
            let root = std::env::temp_dir().join(format!("xs-cli-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            std::fs::create_dir_all(root.join("shared/assets/js")).unwrap();
            std::fs::write(root.join("shared/assets/js/boot.js"), "shared").unwrap();
            std::fs::create_dir_all(&root).unwrap();
            std::fs::write(root.join("debug.html"), "<link rel=\"rust\" />").unwrap();
            Fixture { root }
        }

        /// The directories an asset is looked up in, for a page in `page`
        /// and a shared tree beside it.
        fn directories(&self, page: &str) -> Vec<PathBuf> {
            vec![self.root.join(page), self.root.join("shared")]
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn an_asset_is_found_in_a_later_directory() {
        // The debug case: a page kept somewhere with no assets of its own says
        // nothing about where the bindings it asks for are, so the shared tree
        // has to still be searched.
        let fixture = Fixture::new("later");
        assert_eq!(
            resolve(&fixture.directories("."), "assets/js/boot.js"),
            Some(fixture.root.join("shared/assets/js/boot.js"))
        );
    }

    #[test]
    fn the_first_directory_wins() {
        // A project that overrides the shipped bindings must not be shadowed by
        // them.
        let fixture = Fixture::new("first");
        std::fs::create_dir_all(fixture.root.join("assets/js")).unwrap();
        std::fs::write(fixture.root.join("assets/js/boot.js"), "own").unwrap();
        assert_eq!(
            resolve(&fixture.directories("."), "assets/js/boot.js"),
            Some(fixture.root.join("assets/js/boot.js"))
        );
    }

    #[test]
    fn a_missing_asset_is_still_a_missing_asset() {
        let fixture = Fixture::new("missing");
        assert!(resolve(&fixture.directories("."), "assets/js/nope.js").is_none());
    }

    #[test]
    fn the_page_is_the_root_and_its_own_file_name() {
        // Anything else serves the file as a plain asset, and the page as it
        // sits on disk carries no loader, so a link to it would boot nothing.
        let html = Path::new("pages/debug.html");
        assert!(is_page(html, ""));
        assert!(is_page(html, "debug.html"));
        assert!(is_page(html, "pages/debug.html"));
        assert!(!is_page(html, "index.html"));
        assert!(!is_page(html, "assets/js/boot.js"));
    }

    #[test]
    fn nothing_resolves_outside_the_directories() {
        let fixture = Fixture::new("escape");
        assert!(resolve(&fixture.directories("."), "../shared").is_none());
        assert!(resolve(&fixture.directories("."), "assets/js/../../../etc").is_none());
    }
}
