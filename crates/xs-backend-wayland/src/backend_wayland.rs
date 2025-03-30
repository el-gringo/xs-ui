use xs_layout::Layout;
use xs_renderer_opengl::RendererOpenGl;

use crate::{config::ConfigWayland, linux_wayland::window};



pub struct BackendWayland {
    width: i32,
    height: i32,
}

impl BackendWayland {
    pub fn new(width: i32, height: i32) -> Self {
        Self { width, height }
    }

    pub fn run<F>(&mut self, generate_scene: F)
    where F: Fn() -> Box<dyn Layout> {
        let config = ConfigWayland {
            window_title: String::from("Test"),
            window_width: self.width,
            window_height: self.height,
            fullscreen: false
        };
        let mut renderer = RendererOpenGl::new(self.width, self.height);
        window(&config, &mut renderer as _, generate_scene);
    }

    pub fn log(&self, _message: &str) {

    }
}
