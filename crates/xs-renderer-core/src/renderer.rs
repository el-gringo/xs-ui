use xs_layout::Layout;

pub trait Renderer {
    fn init(&mut self);
    fn screen_width(&self) -> i32;
    fn screen_height(&self) -> i32;
    fn draw_frame(&mut self, widget_tree: Box<dyn Layout>);
}
