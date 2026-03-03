use crate::response::{HttpHeaders, HttpResponse};
use xs_js_sys::{Closure, fetch};
use xs_url::url::Url;

#[allow(unused)]
pub struct HttpClient {
    url: Url,
    headers: HttpHeaders,
}

impl HttpClient {
    pub fn new(url: impl Into<Url>) -> Self {
        let url = url.into();
        Self {
            url,
            headers: HttpHeaders::new(),
        }
    }

    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn headers(mut self, headers: HttpHeaders) -> Self {
        self.headers = headers;
        self
    }

    pub fn get(self, on_response: impl FnMut(String) + Send + 'static) {
        self.send("GET".into(), None, on_response)
    }
    pub fn post(self, body: Option<&[u8]>, on_response: impl FnMut(String) + Send + 'static) {
        self.send("POST".into(), body, on_response)
    }
    pub fn put(self, body: Option<&[u8]>, on_response: impl FnMut(String) + Send + 'static) {
        self.send("PUT".into(), body, on_response)
    }
    pub fn patch(self, body: Option<&[u8]>, on_response: impl FnMut(String) + Send + 'static) {
        self.send("PATCH".into(), body, on_response)
    }
    pub fn delete(self, body: Option<&[u8]>, on_response: impl FnMut(String) + Send + 'static) {
        self.send("DELETE".into(), body, on_response)
    }

    fn send(
        mut self,
        method: &str,
        body: Option<&[u8]>,
        on_response: impl FnMut(String) + Send + 'static
    ) {
        fetch(self.url.into(), None, Closure::new(on_response))
    }
}
