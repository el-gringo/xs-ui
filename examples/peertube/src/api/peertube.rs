use std::{collections::BTreeMap, fmt};

use nanoserde::DeJson;

use crate::api::rest_api::RestApi;

pub const PEERTUBE_BASE_URL: &str = "https://peertube.fr";
const PEERTUBE_API_PREFIX: &str = "/api/v1";

#[derive(Clone, Debug)]
pub enum ApiError {
    Status(u16),
    Decode { what: &'static str, reason: String },
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiError::Status(code) => write!(f, "the server answered {code}"),
            ApiError::Decode { what, reason } => write!(f, "{what} could not be read: {reason}"),
        }
    }
}

#[derive(DeJson, Clone, Debug, Default)]
pub struct Instance {
    pub name: String,
    #[nserde(rename = "shortDescription")]
    pub short_description: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Category {
    pub id: u32,
    pub label: String,
}

impl Category {
    fn from_map(map: BTreeMap<String, String>) -> Vec<Category> {
        let mut categories: Vec<Category> = map
            .into_iter()
            .filter_map(|(id, label)| {
                Some(Category {
                    id: id.parse().ok()?,
                    label,
                })
            })
            .collect();
        categories.sort_by_key(|category| category.id);
        categories
    }
}

#[derive(Default, DeJson, Clone, Debug)]
pub struct Video {
    pub name: String,
    #[nserde(rename = "thumbnailPath")]
    pub thumbnail_path: String,
}

impl Video {
    pub fn thumbnail_url(&self) -> String {
        PEERTUBE_BASE_URL.to_owned() + &self.thumbnail_path
    }
}

#[derive(Debug, DeJson)]
struct ConfigResponse {
    instance: Instance,
}

#[derive(Debug, DeJson)]
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

    pub fn get_instance(
        &self,
        on_result: impl FnOnce(Result<Instance, ApiError>) + Send + 'static,
    ) {
        let mut on_result = Some(on_result);
        self.get("/config", "the instance configuration", move |config| {
            let Some(deliver) = on_result.take() else {
                return;
            };
            deliver(config.map(|config: ConfigResponse| config.instance));
        });
    }

    pub fn get_categories(
        &self,
        on_result: impl FnOnce(Result<Vec<Category>, ApiError>) + Send + 'static,
    ) {
        let mut on_result = Some(on_result);
        self.get("/videos/categories", "the video categories", move |map| {
            let Some(deliver) = on_result.take() else {
                return;
            };
            deliver(map.map(Category::from_map));
        });
    }

    pub fn get_videos(
        &self,
        offset: u32,
        count: u32,
        on_result: impl FnOnce(Result<Vec<Video>, ApiError>) + Send + 'static,
    ) {
        let mut on_result = Some(on_result);
        self.get(
            &format!("/videos?start={offset}&count={count}"),
            "the video list",
            move |list| {
                let Some(deliver) = on_result.take() else {
                    return;
                };
                deliver(list.map(|list: VideoListResponse| list.data));
            },
        );
    }

    fn get<T: DeJson>(
        &self,
        path: &str,
        what: &'static str,
        on_result: impl FnOnce(Result<T, ApiError>) + Send + 'static,
    ) {
        let mut on_result = Some(on_result);
        self.api.get(path, move |response| {
            let Some(deliver) = on_result.take() else {
                return;
            };
            deliver(decode(response.status_code, &response.body_lossy(), what));
        });
    }
}

fn decode<T: DeJson>(status_code: u16, json: &str, what: &'static str) -> Result<T, ApiError> {
    if !(200..300).contains(&status_code) {
        return Err(ApiError::Status(status_code));
    }
    T::deserialize_json(json).map_err(|reason| ApiError::Decode {
        what,
        reason: reason.to_string(),
    })
}

impl Default for PeerTubeClient {
    fn default() -> Self {
        Self::new()
    }
}
