enum EasingTiming {
    Ease,
    EaseIn,
    EaseInOut,
    EaseOut,
    Linear,
}

pub struct Easing {
    start: f32,
    value: f32,
    target: f32,
    duration: f32,
    elapsed: f32,
    timing: EasingTiming,
}

impl Easing {
    pub fn new(value: f32) -> Self {
        Self {
            start: value,
            value,
            target: value,
            duration: 250.0,
            elapsed: 0.0,
            timing: EasingTiming::Linear,
        }
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn ease(mut self) -> Self {
        self.timing = EasingTiming::Ease;
        self
    }

    pub fn ease_in(mut self) -> Self {
        self.timing = EasingTiming::EaseIn;
        self
    }

    pub fn ease_in_out(mut self) -> Self {
        self.timing = EasingTiming::EaseInOut;
        self
    }

    pub fn ease_out(mut self) -> Self {
        self.timing = EasingTiming::EaseOut;
        self
    }

    pub fn snap_to(&mut self, target: f32) {
        if (self.target - target).abs() > f32::EPSILON {
            self.start = self.value;
            self.target = target;
            self.elapsed = 0.0;
        }
    }

    pub fn tick(&mut self, dt: f32) -> f32 {
        self.elapsed += dt * 1000.0;
        let t = (self.elapsed / self.duration).clamp(0.0, 1.0);
        let eased = match self.timing {
            EasingTiming::Linear => t,
            EasingTiming::EaseIn => ease_in(t),
            EasingTiming::EaseOut => ease_out(t),
            EasingTiming::EaseInOut => ease_in_out(t),
            EasingTiming::Ease => ease_in_out(t),
        };
        self.value = self.start + (self.target - self.start) * eased;
        self.value
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration
    }
}

impl crate::Animate for Easing {
    fn snap_to(&mut self, target: f32) {
        self.snap_to(target);
    }

    fn tick(&mut self, dt: f32) -> f32 {
        self.tick(dt)
    }

    fn value(&self) -> f32 {
        self.value()
    }
}

fn ease_in(t: f32) -> f32 {
    t * t
}

fn ease_out(t: f32) -> f32 {
    t * (2.0 - t)
}

fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        2.0 * t * t
    } else {
        -1.0 + (4.0 - 2.0 * t) * t
    }
}
