use std::io::Write;

use xs_http::{response::{
        HttpHeaders,
        HttpResponse
    },
    server::HttpServer
};

pub fn main() {
    let mut server = HttpServer::listen("127.0.0.1", 8888);
    for request in server.incoming_requests() {
        println!("request: {:?}", request);
        let response = HttpResponse::new(
            200,
            HttpHeaders::new(),
            "test\r\n".as_bytes().to_vec()
        );
        request.respond(response);
    }
}
