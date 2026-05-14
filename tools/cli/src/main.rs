use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use handle_static::handle_static;
use xs_http::{HttpHeaders, HttpResponse};
use xs_websocket::{message::WebSocketMessage, opcode::OpCode};

mod cargo_metadata;
mod handle_static;
mod watcher;

const HOST: &str = "0.0.0.0";
const PORT: u16 = 8080;

pub fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let package = args.first().clone();

    let (tx, rx) = std::sync::mpsc::channel();
    let config = cargo_metadata::get_config();
    let (package, addittional_args) = match package {
        Some(package) => (
            package.to_string(),
            vec!["-p".to_string(), package.to_string()],
        ),
        None => ("peertube".to_string(), vec![]),
    };

    let target_directory = config.target_directory.clone();
    watcher::watch(target_directory.clone(), addittional_args, tx);

    let mut http_server = xs_http::HttpServer::listen(HOST, PORT);

    let listeners = Arc::new(Mutex::new(Vec::new()));
    let listeners_clone = Arc::clone(&listeners);
    println!("Server ready on http://{}:{}/", HOST, PORT);
    std::thread::spawn(move || {
        for request in http_server.incoming_requests() {
            let (method, pathname) = (request.method.as_str(), request.pathname.as_str());
            match (method, pathname) {
                ("GET", "/ws") => {
                    let mut wss = xs_websocket::handshake::handshake(request)
                        .expect("failed to create websocket server");
                    listeners_clone
                        .lock()
                        .unwrap()
                        .push(wss.stream.try_clone().unwrap());
                    std::thread::spawn(move || {
                        for message in wss.incoming_messages() {
                            println!("received message: {:?} {:?}", message, message.op_code);
                        }
                    });
                }
                ("GET", _) => {
                    handle_static(request, &config, &package);
                }
                _ => {
                    let response =
                        HttpResponse::new(404, HttpHeaders::new(), "Not found".as_bytes().to_vec());
                    request.respond(response);
                }
            }
        }
    });

    if let Err(error) = rx.recv() {
        println!("error: {:?}", error);
    }
    while let Ok(wasm_pathname) = rx.recv() {
        for mut listener in listeners.lock().unwrap().iter() {
            let wasm_pathname = wasm_pathname.clone();
            let message = WebSocketMessage::new(OpCode::Text, None, wasm_pathname.into());
            let buffer: Vec<u8> = message.into();
            listener.write_all(&buffer).unwrap();
        }
    }
}
