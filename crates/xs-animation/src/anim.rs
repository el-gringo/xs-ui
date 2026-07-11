pub trait Animate: 'static {
    fn snap_to(&mut self, target: f32);
    fn tick(&mut self, dt: f32) -> f32;
    fn value(&self) -> f32;
    fn clone_box(&self) -> Box<dyn Animate>;
}

impl Clone for Box<dyn Animate> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
