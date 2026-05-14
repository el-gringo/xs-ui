use std::io::Read;
use xs_http::{HttpHeaders, HttpRequest, HttpResponse};

use crate::cargo_metadata::CargoMetadata;

//static INDEX_HTML: &'static str = include_str!("ressource/index.html");
//static SCRIPT_JS: &'static str = include_str!("ressource/script.js");

fn get_content_type(path: &std::path::PathBuf) -> &'static str {
    let extension = match path.extension() {
        None => return "text/plain",
        Some(e) => e,
    };
    match extension.to_str().unwrap() {
        "gif" => "image/gif",
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

pub fn handle_static(request: HttpRequest, config: &CargoMetadata, package: &String) {
    let pathname = match request.pathname.split_once('?') {
        Some((path, _)) => &path[1..],
        None => &request.pathname[1..],
    };
    let path = if pathname == "" {
        &std::path::Path::new("tools/cli/src/html").join("index.html")
    } else if pathname.starts_with("assets/") {
        &std::path::Path::new("tools/cli/src/html").join(pathname)
    } else if pathname == "app.wasm" {
        &std::path::Path::new("tools/cli/src")
            .join(&config.target_directory)
            .join("wasm32-unknown-unknown")
            .join("debug")
            .join(package.to_owned() + ".wasm")
    } else {
        &std::path::Path::new(&config.target_directory).join(pathname)
    };
    let file = std::fs::File::open(path);
    let response = if file.is_ok() && path.is_file() {
        let mut file = file.unwrap();
        let content_length = file
            .metadata()
            .ok()
            .map(|v| v.len() as usize)
            .expect("unable to get file size".into());

        let mut body = Vec::with_capacity(content_length);
        file.read_to_end(&mut body).expect("unable to read file");

        let mut headers = HttpHeaders::new();
        headers.insert(
            "Content-Type".to_string(),
            get_content_type(&path).to_string(),
        );
        headers.insert("Content-Length".to_string(), content_length.to_string());
        let response = HttpResponse::new(200, headers, body);
        response
    } else {
        let mut headers = HttpHeaders::new();
        headers.insert(
            "Content-Type".to_string(),
            "text/plain; charset=utf8".to_string(),
        );
        let response = HttpResponse::new(404, headers, "Not found".into());
        response
    };
    request.respond(response);
}
