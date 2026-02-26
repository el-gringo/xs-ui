use crate::{Application, Render, Renderer};

pub trait Backend {
    fn run(
        &mut self,
        app: &mut Application,
        renderer: &mut Box<dyn Renderer>,
        root: Box<dyn Render>,
    );
    fn performance_now(&self) -> u128;
    fn load_image(&mut self, src: &'static str, texture_index: u32);
    fn new_image(&mut self, array: Vec<u8>, width: u32) -> u32;
}

pub struct BackendDummy;
impl Backend for BackendDummy {
    fn run(
        &mut self,
        _app: &mut Application,
        _renderer: &mut Box<dyn Renderer>,
        _root: Box<dyn Render>,
    ) {
    }
    fn load_image(&mut self, _src: &'static str, _texture_index: u32) {}
    fn new_image(&mut self, _array: Vec<u8>, _width: u32) -> u32 {
        0
    }

    fn performance_now(&self) -> u128 {
        0
    }
}
