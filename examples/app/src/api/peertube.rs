use nanoserde::DeJson;
use xs::StateMutator;

use crate::api::rest_api::{RestApi, body_string};

const PEERTUBE_BASE_URL: &str = "https://video.monsieurbidouille.fr";
const PEERTUBE_API_URL: &str = "https://video.monsieurbidouille.fr/api/v1";

#[derive(Default, DeJson, Clone, Debug)]
pub struct Video {
    pub uuid: String,
    pub name: String,
    #[nserde(rename = "thumbnailPath")]
    pub thumbnail_path: Option<String>,
}

impl Video {
    pub fn thumbnail_url(&self) -> Option<String> {
        self.thumbnail_path
            .as_ref()
            .map(|path| PEERTUBE_BASE_URL.to_owned() + path)
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
            api: RestApi::new(PEERTUBE_API_URL),
        }
    }

    pub fn get_videos(&self, offset: u32, count: u32, mutator: StateMutator<Vec<Video>>)
    {
        let api = RestApi::new(PEERTUBE_API_URL);
        api.get(&format!("/videos?start={offset}&count={count}"), move |response| {
            let json_str = body_string(&response);
            if let Ok(VideoListResponse { data }) = VideoListResponse::deserialize_json(&json_str) {
                mutator.mutate(|videos| {
                    videos.extend(data);
                });
            };
        });
    }
}

impl Default for PeerTubeClient {
    fn default() -> Self {
        Self::new()
    }
}
