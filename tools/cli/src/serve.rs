use std::{
    io::Write,
    net::TcpStream,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU32, Ordering},
    },
};

use crate::{
    handle_static::{Site, handle_static},
    html::Html,
    options::Options,
    watcher::Rebuild,
};
use xs_websocket::{message::WebSocketMessage, opcode::OpCode};

use crate::build::WASM_NAME;

/// The browsers watching the dev server.
type Clients = Arc<Mutex<Vec<TcpStream>>>;

/// Serves the project over http, rebuilding it on change and pushing the new
/// wasm url to every open page.
pub fn run(options: &Options) {
    let package = crate::resolve_package(options);
    let metadata = crate::cargo_metadata::get_config();
    if let Err(error) = Html::read(&options.html) {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
    let site = Arc::new(Site {
        html: options.html.clone(),
        wasm_name: WASM_NAME.to_string(),
        wasm_path: options.wasm_path(&metadata.target_directory),
        dev: !options.release,
    });

    // A second server on the same port is a silent failure, say so instead.
    if let Err(error) = std::net::TcpListener::bind((options.host.as_str(), options.port)) {
        eprintln!(
            "error: unable to bind {}:{}, another server is probably already running \
             (pass --port to use another one)\n{error}",
            options.host, options.port
        );
        std::process::exit(1);
    }

    let rebuilds = options
        .watch
        .then(|| crate::watcher::watch(&options.cargo_args(), WASM_NAME));
    let version = Arc::new(AtomicU32::new(0));
    let clients: Clients = Arc::new(Mutex::new(Vec::new()));

    let mut server = xs_http::HttpServer::listen(&options.host, options.port);
    println!(
        "serving {package} (html: {}) on http://{}:{}/",
        options.html.display(),
        options.host,
        options.port
    );

    let page_version = version.clone();
    let page_clients = clients.clone();
    std::thread::spawn(move || {
        for request in server.incoming_requests() {
            let (method, pathname) = (request.method.as_str(), request.pathname.as_str());
            match (method, pathname) {
                ("GET", "/ws") => {
                    let Some(mut wss) = xs_websocket::handshake::handshake(request) else {
                        eprintln!("websocket handshake failed");
                        continue;
                    };
                    match wss.stream.try_clone() {
                        Ok(stream) => page_clients.lock().unwrap().push(stream),
                        Err(error) => eprintln!("could not keep the socket: {error}"),
                    }
                    std::thread::spawn(move || {
                        for message in wss.incoming_messages() {
                            println!("received message: {:?} {:?}", message, message.op_code);
                        }
                    });
                }
                ("GET", _) => {
                    handle_static(request, &site, page_version.load(Ordering::Relaxed));
                }
                _ => {
                    let response = xs_http::HttpResponse::new(
                        404,
                        xs_http::HttpHeaders::new(),
                        "Not found".as_bytes().to_vec(),
                    );
                    request.respond(response);
                }
            }
        }
    });

    match rebuilds {
        // Blocks forever, telling every connected page about the new wasm.
        Some(rebuilds) => {
            for rebuild in rebuilds {
                version.store(url_version(&rebuild.url).unwrap_or(1), Ordering::Relaxed);
                broadcast(&clients, &rebuild);
            }
        }
        None => loop {
            std::thread::sleep(std::time::Duration::from_secs(3600));
        },
    }
}

/// `app.wasm?v=3` -> `Some(3)`.
fn url_version(url: &str) -> Option<u32> {
    url.rsplit_once("v=").and_then(|(_, v)| v.parse().ok())
}

fn broadcast(clients: &Clients, rebuild: &Rebuild) {
    let message = WebSocketMessage::new(OpCode::Text, None, rebuild.url.clone().into());
    let buffer: Vec<u8> = message.into();
    // A browser that went away is dropped instead of failing every rebuild.
    let mut clients = clients.lock().unwrap();
    clients.retain_mut(|client| {
        client
            .write_all(&buffer)
            .and_then(|_| client.flush())
            .is_ok()
    });
}
