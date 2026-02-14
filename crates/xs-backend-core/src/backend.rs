use crate::Render;

pub trait Backend {
    fn run(&mut self, root: Box<dyn Render>);
    fn load_image(&mut self, src: &'static str);
}

pub struct BackendDummy;
impl Backend for BackendDummy {
    fn run(&mut self, mut _root: Box<dyn Render>) {}
    fn load_image(&mut self, _src: &'static str) {}
}
