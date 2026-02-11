use crate::Render;

pub trait Backend {
    fn run(&mut self, root: Box<dyn Render>);
}

pub struct BackendDummy;
impl Backend for BackendDummy {
    fn run(&mut self, mut _root: Box<dyn Render>) {}
}
