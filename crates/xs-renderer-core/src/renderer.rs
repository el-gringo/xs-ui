use xs_layout::Layout;

pub trait Renderer {
    fn init(&mut self, egl_get_proc_address: unsafe extern "C" fn(*const i8) -> Option<unsafe extern "C" fn()>);
    fn draw_frame(&mut self, widget_tree: Box<dyn Layout>);
}
