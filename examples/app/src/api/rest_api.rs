use xs_http::HttpClient;

pub struct RestApi {
    base_url: String,
    client: HttpClient,
}

impl RestApi {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            client: HttpClient::new(base_url),
        }
    }

    pub fn get(&self, path: &str) -> xs_http::response::HttpResponse {
        let url = format!("{}{}", self.base_url, path);
        HttpClient::new(url).get()
    }

    pub fn post(&self, path: &str, body: Option<&[u8]>) -> xs_http::response::HttpResponse {
        let url = format!("{}{}", self.base_url, path);
        HttpClient::new(url).post(body)
    }
}

pub fn body_string(response: &xs_http::response::HttpResponse) -> String {
    String::from_utf8_lossy(&response.body).to_string()
}
