pub trait Animate: std::any::Any + 'static {
    fn tick(&mut self, dt: f32) -> f32;
    fn reset(&mut self);
    fn value(&self) -> f32;
    fn is_finished(&self) -> bool;
    fn clone_box(&self) -> Box<dyn Animate>;

    fn same_config(&self, other: &dyn Animate) -> bool {
        std::any::Any::type_id(self) == std::any::Any::type_id(other)
    }
}

impl Clone for Box<dyn Animate> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
