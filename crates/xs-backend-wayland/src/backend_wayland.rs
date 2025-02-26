use xs_layout::Layout;
use xs_renderer_opengl::RendererOpenGl;

use crate::config::ConfigWayland;

pub struct BackendWayland;
impl BackendWayland {

    pub fn new() -> Self {
        Self
    }

    pub fn render(&mut self, widget_tree: Box<dyn Layout>) {
        let _config = ConfigWayland {
            window_title: String::from("Test"),
            window_width: 960,
            window_height: 540,
            fullscreen: false,
            use_fallback_decorations: true
        };
        let mut renderer = RendererOpenGl::new();
        renderer.render(widget_tree);
    }

    pub fn log(&self, _message: &str) {

    }
}
