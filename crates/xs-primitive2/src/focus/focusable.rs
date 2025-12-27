pub trait Focusable {
    fn on_focus<F>(&mut self, callback: F) where F: Fn();
    fn on_blur<F>(&mut self, callback: F) where F: Fn();
}
