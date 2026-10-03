use nanoserde::DeJson;

use crate::api::rest_api::RestApi;

const PEERTUBE_BASE_URL: &str = "https://video.monsieurbidouille.fr";
const PEERTUBE_API_PREFIX: &str = "/api/v1";

#[derive(Default, DeJson, Clone, Debug)]
pub struct Video {
    //pub uuid: String,
    pub name: String,
    #[nserde(rename = "thumbnailPath")]
    pub thumbnail_path: String,
}

impl Video {
    pub fn thumbnail_url(&self) -> String {
        PEERTUBE_BASE_URL.to_owned() + &self.thumbnail_path
    }
}

#[derive(DeJson)]
struct VideoListResponse {
    data: Vec<Video>,
}

pub struct PeerTubeClient {
    api: RestApi,
}

impl PeerTubeClient {
    pub fn new() -> Self {
        Self {
            api: RestApi::new(&(PEERTUBE_BASE_URL.to_owned() + PEERTUBE_API_PREFIX)),
        }
    }

    pub fn get_videos(
        self,
        offset: u32,
        count: u32,
        on_result: impl FnOnce(Vec<Video>) + Send + 'static,
    ) {
        let mut on_result = Some(on_result);
        self.api.get(
            &format!("/videos?start={offset}&count={count}"),
            move |response| {
                let json = response.body_lossy();
                if let Ok(VideoListResponse { data }) = VideoListResponse::deserialize_json(&json) {
                    if let Some(on_result) = on_result.take() {
                        on_result(data);
                    }
                };
            },
        );
    }
}

impl Default for PeerTubeClient {
    fn default() -> Self {
        Self::new()
    }
}
