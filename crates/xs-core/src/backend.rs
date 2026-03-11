
use xs_layout::TextProperties;

use crate::{AppContext, Application, Render, Renderer, TextSystem};

pub enum BackendResponse {
    /// image src
    /// image handle (js)
    /// bytes (native)
    /// width
    /// height
    ImageLoaded(String, i32, Vec<u8>, u32, u32),
    /// text, text-properties
    TextRendered(String, TextProperties, Vec<u8>, u32, u32),
}

pub enum BackendRequest {
    /// src url
    LoadImage(String),
    Text(String, TextProperties),
}

pub trait Backend {
    fn run(
        &mut self,
        app: Application,
        app_context: AppContext,
        renderer: Box<dyn Renderer>,
        text_system: Box<dyn TextSystem>,
        root: Box<dyn Render>,
    );
    fn performance_now(&self) -> u128;
}

const BACKEND_DUMMY_WARNING: &str =
    "BackendDummy::load_image: thre right backend is not loaded corectly";
pub struct BackendDummy;
impl Backend for BackendDummy {
    fn run(
        &mut self,
        _app: Application,
        _app_context: AppContext,
        _renderer: Box<dyn Renderer>,
        _text_system: Box<dyn TextSystem>,
        _root: Box<dyn Render>,
    ) {
    }
    fn performance_now(&self) -> u128 {
        panic!("{}", BACKEND_DUMMY_WARNING);
    }
}
