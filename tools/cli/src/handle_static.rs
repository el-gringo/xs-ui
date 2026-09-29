use std::io::Read;
use std::path::{Path, PathBuf};
use xs_http::{HttpHeaders, HttpRequest, HttpResponse};

use crate::cargo_metadata::CargoMetadata;

/// Static files (index.html, boot.js, the bindings) are owned by the binding
/// system crate, so the CLI and `serve.sh` always serve the same ones.
const HTML_DIR: &str = "crates/xs-js-sys/html";

/// The wasm is always served as `app.wasm`, the name the html requests.
const WASM_PATHNAME: &str = "app.wasm";

/// Placeholder the html uses to build the wasm url, replaced when serving.
const WASM_PLACEHOLDER: &str = "{wasm-pathname}";
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
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        "js" => "text/javascript; charset=utf8",
        _ => "text/plain; charset=utf8",
    }
}

fn not_found() -> HttpResponse {
    let mut headers = HttpHeaders::new();
    headers.insert(
        "Content-Type".to_string(),
        "text/plain; charset=utf8".to_string(),
    );
    HttpResponse::new(404, headers, "Not found".into())
}

/// Looks the request up in the binding system html first, then in the cargo
/// target directory (for anything the build emitted, e.g. debug artifacts).
fn resolve(pathname: &str, html_dir: &Path, target_dir: &Path, package: &str) -> Option<PathBuf> {
    if pathname.split('/').any(|segment| segment == "..") {
        return None;
    }
    let path = match pathname {
        "" => html_dir.join("index.html"),
        WASM_PATHNAME => target_dir
            .join("wasm32-unknown-unknown")
            .join("debug")
            .join(format!("{package}.wasm")),
        _ => {
            let from_html = html_dir.join(pathname);
            if from_html.is_file() {
                from_html
            } else {
                target_dir.join(pathname)
            }
        }
    };
    path.is_file().then_some(path)
}

fn read_file(path: &Path) -> Option<Vec<u8>> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut body = Vec::new();
    file.read_to_end(&mut body).ok()?;
    Some(body)
}

pub fn handle_static(request: HttpRequest, config: &CargoMetadata, package: &String) {
    let pathname = match request.pathname.split_once('?') {
        Some((path, _)) => &path[1..],
        None => &request.pathname[1..],
    };

    let html_dir = Path::new(HTML_DIR);
    let response = match resolve(
        pathname,
        html_dir,
        Path::new(&config.target_directory),
        package,
    ) {
        Some(path) => match read_file(&path) {
            Some(mut body) => {
                if path == html_dir.join("index.html") {
                    let index = String::from_utf8_lossy(&body).replace(WASM_PLACEHOLDER, "app");
                    body = index.into_bytes();
                }
                let mut headers = HttpHeaders::new();
                headers.insert(
                    "Content-Type".to_string(),
                    get_content_type(&path).to_string(),
                );
                headers.insert("Content-Length".to_string(), body.len().to_string());
                HttpResponse::new(200, headers, body)
            }
            None => not_found(),
        },
        None => not_found(),
    };
    request.respond(response);
}
