use std::io::{BufWriter, Write};

use xs_http::{
    client::HttpRequest,
    digest::{Sha1, base64_encode},
    response::{HttpHeaders, HttpResponse},
};

use crate::{message::WebSocketMessage, server::WebSocketServer};

fn convert_key(key: &str) -> String {
    let hash = key.to_owned() + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
    let mut sha1 = Sha1::new();
    sha1.update(hash.as_bytes());
    let out_bytes = sha1.finalise();
    base64_encode(&out_bytes)
}

fn create_handshake_response(key: &str) -> HttpResponse {
    let mut headers = HttpHeaders::new();
    headers.insert("Upgrade".to_string(), "websocket".to_string());
    headers.insert("Connection".to_string(), "Upgrade".to_string());
    headers.insert("Sec-WebSocket-Accept".to_string(), convert_key(key));
    headers.insert("Sec-WebSocket-Protocol".to_string(), "ping".to_string());
    HttpResponse::new(101, headers, Vec::new())
}

pub fn handshake(request: HttpRequest) -> Option<WebSocketServer> {
    let key = request.headers.get("sec-websocket-key");
    if let Some(key) = key {
        let stream = request.stream.try_clone().unwrap();
        let response = create_handshake_response(key.as_str());
        request.respond(response);
        let wss = WebSocketServer::with_stream(stream);
        let message = WebSocketMessage::new(
            crate::opcode::OpCode::Text,
            None,
            "Welcome".as_bytes().to_vec(),
        );
        let mut writer = BufWriter::new(wss.stream.try_clone().unwrap());
        let mut buffer: Vec<u8> = message.into();
        writer.write_all(&mut buffer).unwrap();
        return Some(wss);
    }
    None
}

#[cfg(test)]
mod tests {
    const KEY: &str = "dGhlIHNhbXBsZSBub25jZQ==";
    const ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

    use crate::handshake::convert_key;

    #[test]
    fn it_can_convert_websocket_key() {
        let actual = convert_key(KEY);
        let expected = ACCEPT;
        assert_eq!(actual, expected);
    }
}
