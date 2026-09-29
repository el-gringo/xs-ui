use std::io::Read;
use std::path::{Path, PathBuf};
use xs_http::{HttpHeaders, HttpRequest, HttpResponse};

use crate::html::Html;

/// Everything the server knows about the app it serves.
pub struct Site {
    /// The html file, read again on every request so editing it needs no
    /// restart.
    pub html: PathBuf,
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

    /// Directory the html lives in, where its assets are served from.
    pub fn directory(&self) -> &Path {
        self.html.parent().unwrap_or(Path::new("."))
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

/// Serves `/` with the wasm loader injected, the wasm itself, and the assets
/// the html refers to.
pub fn handle_static(request: HttpRequest, site: &Site, version: u32) {
    let pathname = match request.pathname.split_once('?') {
        Some((path, _)) => &path[1..],
        None => &request.pathname[1..],
    };

    let response = if pathname.is_empty() {
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
        match resolve(site, pathname) {
            Some(path) => match read_file(&path) {
                Some(body) => respond(200, get_content_type(&path), body),
                None => not_found(),
            },
            None => not_found(),
        }
    };

    request.respond(response);
}

/// Looks a request up next to the html file, so `assets/js/boot.js` resolves
/// the way the page asks for it.
fn resolve(site: &Site, pathname: &str) -> Option<PathBuf> {
    if pathname.split('/').any(|segment| segment == "..") {
        return None;
    }
    let path = site.directory().join(pathname);
    path.is_file().then_some(path)
}
