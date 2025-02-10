
use xs_layout::Layout;
use xs_renderer_webgl::WebGLRenderer;
use crate::bindings::{self, console_log};


//pub struct EventLoop;

/*
use std::{future::Future, pin::Pin, task::{Context, Poll}};
#[derive(Default)]
pub struct FutureFrame {
    done: bool,
}

impl Future for FutureFrame {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _context: &mut Context) -> Poll<Self::Output> {
        if self.done {
            // We were told to step, meaning this future gets destroyed and we run
            // the main future until we call next_frame again and end up in this poll
            // function again.
            Poll::Ready(())
        } else {
            self.done = true;
            Poll::Pending
        }
    }
}
*/

pub struct BackendBrowser {
    renderer: WebGLRenderer
}

impl BackendBrowser {
    pub fn new() -> Self {
        unsafe {
            bindings::init_canvas();
        }
        let renderer = WebGLRenderer::new();
        Self {
            renderer
        }
    }

    pub fn render(&mut self, widget_tree: Box<dyn Layout>) {
        self.renderer.render(widget_tree);
    }

    pub fn get_time(self) -> f64 {
        unsafe {
            bindings::now()
        }
    }

    pub fn log(&self, message: &str) {
        let bytes = message.as_bytes();
        unsafe {
            console_log(bytes.as_ptr() as u32, bytes.len() as u32);
        }
    }
}
