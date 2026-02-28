use std::sync::mpsc::{Receiver, Sender};

use crate::{AppContext, Application, Render, Renderer};

pub enum BackendMessage {
    ImageLoaded(String, Vec<u8>, u32, u32),
}

pub trait Backend {
    fn run(
        &mut self,
        app: &mut Application,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        root: Box<dyn Render>,
    );
    fn performance_now(&self) -> u128;
    fn load_image(&mut self, src: String, tx: Sender<BackendMessage>);
    fn new_image(&mut self, rgba: Vec<u8>, width: u32, height: u32) -> u32;
    fn read_messages(
        &mut self,
        app_context: &mut AppContext,
        renderer: &mut Box<dyn Renderer>,
        tx: Sender<BackendMessage>,
        rx: &Receiver<BackendMessage>,
    );
}

pub struct BackendDummy;
impl Backend for BackendDummy {
    fn run(
        &mut self,
        _app: &mut Application,
        _app_context: &mut AppContext,
        _renderer: &mut Box<dyn Renderer>,
        _root: Box<dyn Render>,
    ) {
    }
    fn load_image(&mut self, _src: String, _tx: Sender<BackendMessage>) {
        panic!("BackendDummy::load_image: thre right backend is not loaded corectly");
    }
    fn new_image(&mut self, _rgba: Vec<u8>, _width: u32, _height: u32) -> u32 {
        panic!("BackendDummy::load_image: thre right backend is not loaded corectly");
    }

    fn performance_now(&self) -> u128 {
        panic!("BackendDummy::load_image: thre right backend is not loaded corectly");
    }
    fn read_messages(
        &mut self,
        _app_context: &mut AppContext,
        _renderer: &mut Box<dyn Renderer>,
        _tx: Sender<BackendMessage>,
        _rx: &Receiver<BackendMessage>,
    ) {
    }
}
