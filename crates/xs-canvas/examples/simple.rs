use xs_canvas::Canvas;
use xs_gl_sys::{GL_COLOR_BUFFER_BIT, glClear, glClearColor, glViewport};

pub fn main() {
    Canvas::new().run(|_ctx| {
        |_ctx| {
            glViewport(0, 0, 960, 540);
            glClear(GL_COLOR_BUFFER_BIT);
            glClearColor(1., 0., 0., 0.);
        }
    });
}
