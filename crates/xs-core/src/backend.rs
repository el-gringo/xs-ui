use std::sync::mpsc::Sender;

use crate::{AppConfig, AppContext};

pub enum BackendResponse {
    /// image src
    /// image handle (js)
    /// bytes (native)
    /// width
    /// height
    ImageLoaded(String, i32, Vec<u8>, u32, u32),
}

pub enum BackendRequest {
    /// src url
    LoadImage(String),
}

pub trait EventLoop {
    fn initialize(&mut self, ctx: &mut AppContext);
    fn frame(&mut self, ctx: &mut AppContext);
}

pub trait Backend {
    fn run(&mut self, app: &AppConfig, app_context: AppContext, event_loop: Box<dyn EventLoop>);
    fn load_image(&mut self, _src: String, _tx: Sender<BackendResponse>) {}
}

pub struct BackendDummy;
impl Backend for BackendDummy {
    fn run(
        &mut self,
        _app_config: &AppConfig,
        _app_context: AppContext,
        _event_loop: Box<dyn EventLoop>,
    ) {
    }

    fn load_image(&mut self, _src: String, _tx: Sender<BackendResponse>) {}
}
