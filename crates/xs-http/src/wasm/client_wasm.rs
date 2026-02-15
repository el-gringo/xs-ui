use xs_url::url::Url;
use crate::response::HttpHeaders;

 #[allow(unused)]
pub struct HttpClient {
    url: Url,
    headers: HttpHeaders,
}
