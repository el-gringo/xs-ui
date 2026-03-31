use std::rc::Rc;

use crate::{AppContext, Application, Render, Renderer};

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

pub trait Backend {
    fn run(
        &mut self,
        app: Application,
        app_context: AppContext,
        renderer: Box<dyn Renderer>,
        root: Rc<dyn Render>,
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
        _root: Rc<dyn Render>,
    ) {
    }
    fn performance_now(&self) -> u128 {
        panic!("{}", BACKEND_DUMMY_WARNING);
    }
}
