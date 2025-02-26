use xs_backend::Backend;

mod app;

pub fn main() {
    let mut backend = Backend::new();
    let widget_tree = app::sidebar();
    backend.log("test");
    backend.render(widget_tree);
}
