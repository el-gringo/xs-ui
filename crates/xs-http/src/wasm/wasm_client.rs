use crate::{headers::HttpHeaders, response::HttpResponse};
use xs_js_sys::{Closure, fetch};
use xs_url::url::Url;

pub struct HttpClient {
    pub(crate) headers: HttpHeaders,
}

impl Default for HttpClient {
    fn default() -> Self {
        let mut headers: HttpHeaders = Default::default();
        // Some sites requires a User-Agent to avoid bots
        headers.insert("User-Agent".into(), "XS-Http".into());
        Self { headers }
    }
}

impl HttpClient {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }
    pub fn headers(mut self, headers: HttpHeaders) -> Self {
        self.headers = headers;
        self
    }
    pub fn get(
        self,
        url: impl Into<Url>,
        on_response: impl FnMut(HttpResponse) + 'static,
    ) -> FakeJoinHandle {
        self.send("GET", url, None, on_response)
    }
    pub fn post(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + 'static,
    ) -> FakeJoinHandle {
        self.send("POST", url, body, on_response)
    }
    pub fn put(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + 'static,
    ) -> FakeJoinHandle {
        self.send("PUT", url, body, on_response)
    }
    pub fn patch(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + 'static,
    ) -> FakeJoinHandle {
        self.send("PATCH", url, body, on_response)
    }
    pub fn delete(
        self,
        url: impl Into<Url>,
        body: Option<&[u8]>,
        on_response: impl FnMut(HttpResponse) + 'static,
    ) -> FakeJoinHandle {
        self.send("DELETE", url, body, on_response)
    }
    pub fn send(
        &self,
        method: impl Into<String>,
        url: impl Into<Url>,
        _body: Option<&[u8]>,
        mut on_response: impl FnMut(HttpResponse) + 'static,
    ) -> FakeJoinHandle {
        let _method = method.into();
        let url = url.into();
        let wrapped = Closure::new(move |body: Vec<u8>| {
            let response = HttpResponse::new(200, Default::default(), body);
            on_response(response);
        });
        let url_str = url.to_string();
        fetch(&url_str, None, wrapped);
        FakeJoinHandle
    }
}

/// Stands in for the thread handle the native client returns.
///
/// There is no thread to wait for here: the response arrives through the
/// callback that `send` was given. The `Result` is kept so that call sites
/// compile the same way on both platforms, but it cannot be an error.
pub struct FakeJoinHandle;

impl FakeJoinHandle {
    pub fn join(&self) -> Result<(), std::convert::Infallible> {
        Ok(())
    }
}
