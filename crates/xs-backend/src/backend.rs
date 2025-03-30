trait Backend {
    pub fn run<F>(&mut self, generate_scene: F)
        where F: Fn() -> Box<dyn Layout>;
}
