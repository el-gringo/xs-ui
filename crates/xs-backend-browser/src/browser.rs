
use xs_layout::Layout;
use xs_renderer_core::Renderer;
use xs_renderer_webgl::RendererWebGl;
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

fn render<F>(renderer: &mut impl Renderer, generate_scene: F)
where F: Fn() -> Box<dyn Layout>{
    let widget_tree = generate_scene();
    renderer.draw_frame(widget_tree);
}

#[allow(dead_code)]
pub struct BackendBrowser {
    renderer: RendererWebGl
}

impl BackendBrowser {
    pub fn new(width: i32, height: i32) -> Self {
        unsafe {
            bindings::init_canvas();
        }
        let renderer = RendererWebGl::new(width, height);
        Self {
            renderer
        }
    }

    pub fn run<F>(&mut self, generate_scene: F)
    where F: Fn() -> Box<dyn Layout> {
        render(&mut self.renderer, generate_scene);
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
