use std::rc::Rc;

use xs::State;

use crate::api::peertube::{ApiError, Category, Instance, PeerTubeClient, Video};

#[derive(Clone, Default)]
pub struct Loaded {
    pub instance: Option<Instance>,
    pub categories: Option<Vec<Category>>,
    pub videos: Option<Rc<Vec<Video>>>,
    pub error: Option<String>,
}

pub struct Interface {
    pub instance: Instance,
    pub categories: Vec<Category>,
    pub videos: Rc<Vec<Video>>,
}

impl Loaded {
    pub fn opened(&self) -> Option<Interface> {
        Some(Interface {
            instance: self.instance.clone()?,
            categories: self.categories.clone()?,
            videos: self.videos.clone()?,
        })
    }
}

#[derive(Clone)]
pub struct Loader {
    state: State<Loaded>,
}

impl Loader {
    pub fn new(state: State<Loaded>) -> Self {
        Self { state }
    }

    pub fn load(&self, video_count: u32) {
        let client = PeerTubeClient::new();

        let instance = self.clone();
        client.get_instance(move |result| {
            instance.publish(|loaded| {
                loaded.instance = Some(result?);
                Ok(())
            })
        });

        let categories = self.clone();
        client.get_categories(move |result| {
            categories.publish(|loaded| {
                loaded.categories = Some(result?);
                Ok(())
            })
        });

        let videos = self.clone();
        client.get_videos(0, video_count, move |result| {
            videos.publish(|loaded| {
                loaded.videos = Some(Rc::new(result?));
                Ok(())
            })
        });
    }

    fn publish(&self, merge: impl FnOnce(&mut Loaded) -> Result<(), ApiError>) {
        let mut loaded = self.state.read().clone();
        if let Err(error) = merge(&mut loaded)
            && loaded.error.is_none()
        {
            loaded.error = Some(error.to_string());
        }
        self.state.set(loaded);
    }
}
