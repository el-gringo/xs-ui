use nanoserde::DeJson;

use crate::api::rest_api::{RestApi, body_string};

const PEERTUBE_BASE_URL: &str = "https://video.monsieurbidouille.fr";
const PEERTUBE_API_URL: &str = "https://video.monsieurbidouille.fr/api/v1";

#[derive(Default, DeJson, Clone, Debug)]
pub struct Video {
    pub uuid: String,
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
            api: RestApi::new(PEERTUBE_API_URL),
        }
    }

    pub fn get_videos(
        &self,
        offset: u32,
        count: u32,
        on_result: impl FnOnce(Vec<Video>) + Send + 'static,
    ) {
        let api = RestApi::new(PEERTUBE_API_URL);
        let mut on_result = Some(on_result);
        api.get(
            &format!("/videos?start={offset}&count={count}"),
            move |response| {
                let json_str = body_string(&response);
                if let Ok(VideoListResponse { data }) =
                    VideoListResponse::deserialize_json(&json_str)
                {
                    let new_videos: Vec<Video> = data
                        .into_iter()
                        .filter(|v| {
                            v.thumbnail_path.ends_with(".jpg")
                                || v.thumbnail_path.ends_with(".jpeg")
                        })
                        .collect();
                    if let Some(on_result) = on_result.take() {
                        on_result(new_videos);
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
