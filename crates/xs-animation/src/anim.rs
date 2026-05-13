pub trait Animate {
    fn snap_to(&mut self, target: f32);
    fn tick(&mut self, dt: f32) -> f32;
    fn value(&self) -> f32;
}
