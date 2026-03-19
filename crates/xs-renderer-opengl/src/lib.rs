mod shader;

mod renderer_opengl;
pub use renderer_opengl::*;
pub use xs_gl_sys::{GL_COLOR_BUFFER_BIT, glClear, glClearColor, glFlush};
