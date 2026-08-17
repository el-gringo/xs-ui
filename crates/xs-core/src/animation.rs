use xs_animation::Animate;

/// Persistent animation state shared by `Context::animate` (scalar
/// interpolation) and element-level animations (`with_animation`).
pub struct AnimationState {
    /// Pristine copy of the animation configuration, kept to detect
    /// configuration changes and re-trigger the animation.
    template: Box<dyn Animate>,
    /// The running animation instance, ticked each frame.
    animate: Box<dyn Animate>,
    last_tick: u128,
}

impl AnimationState {
    pub fn new(animate: impl Animate, now: u128) -> Self {
        Self::new_boxed(Box::new(animate), now)
    }

    pub fn new_boxed(animate: Box<dyn Animate>, now: u128) -> Self {
        let template = animate.clone_box();
        Self {
            template,
            animate,
            last_tick: now,
        }
    }

    /// Re-triggers the animation if `incoming` config differs from the one
    /// currently held (e.g. a changed duration/repeat/direction).
    pub fn retrigger_if_changed(&mut self, incoming: Box<dyn Animate>, now: u128) {
        if !self.template.same_config(&*incoming) {
            self.animate = incoming.clone_box();
            self.template = incoming;
            self.last_tick = now;
        }
    }

    /// Restarts the running animation from its initial state.
    pub fn restart(&mut self, now: u128) {
        self.animate.reset();
        self.last_tick = now;
    }

    /// Advances the animation by the wall-clock time elapsed since the last
    /// tick and returns the current progress (0..1). Skips ticking when the
    /// frame gap is longer than a second so suspended apps don't jump.
    pub fn tick(&mut self, now: u128) -> f32 {
        let elapsed_ms = now.saturating_sub(self.last_tick);
        let dt = elapsed_ms as f32 / 1000.0;
        if elapsed_ms > 0 && elapsed_ms < 1000 {
            self.animate.tick(dt);
        }
        self.last_tick = now;
        self.animate.value()
    }

    pub fn value(&self) -> f32 {
        self.animate.value()
    }

    pub fn is_finished(&self) -> bool {
        self.animate.is_finished()
    }
}
