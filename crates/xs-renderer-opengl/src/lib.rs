mod shader;
mod mesh;

mod renderer_opengl;
pub use renderer_opengl::*;
pub use xs_gl_sys::{glClear, glClearColor, glFlush, GL_COLOR_BUFFER_BIT};
