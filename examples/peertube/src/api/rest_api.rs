use xs_http::{HttpClient, response::HttpResponse};

pub struct RestApi {
    base_url: String,
    client: HttpClient,
}

impl RestApi {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: HttpClient::new(),
        }
    }

    pub fn get(self, path: &str, on_response: impl FnMut(HttpResponse) + Send + 'static) {
        let url = format!("{}{}", self.base_url, path);
        let _ = self.client.get(url, on_response);
    }
}
