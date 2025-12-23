use xs_layout::Layout;

pub trait Renderer {
    fn init(&mut self);
    fn draw_frame(&mut self, widget_tree: Box<dyn Layout>);
}
