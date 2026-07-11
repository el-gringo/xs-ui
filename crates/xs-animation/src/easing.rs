use crate::AnimationDirection;

#[derive(Clone, Copy)]
enum EasingTiming {
    Ease,
    EaseIn,
    EaseInOut,
    EaseOut,
    Linear,
}

#[derive(Clone)]
pub struct Easing {
    start: f32,
    value: f32,
    target: f32,
    duration: f32,
    elapsed: f32,
    timing: EasingTiming,
    repeat: bool,
    direction: AnimationDirection,
    forward: bool,
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
            repeat: false,
            direction: AnimationDirection::Normal,
            forward: true,
        }
    }

    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    pub fn direction(mut self, direction: AnimationDirection) -> Self {
        self.direction = direction;
        match direction {
            AnimationDirection::Normal | AnimationDirection::Alternate => self.forward = true,
            AnimationDirection::Reverse | AnimationDirection::AlternateReverse => self.forward = false,
        }
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
        if self.repeat && self.elapsed >= self.duration {
            self.elapsed %= self.duration;
            self.advance_direction();
        }
        let t = if self.forward {
            (self.elapsed / self.duration).clamp(0.0, 1.0)
        } else {
            1.0 - (self.elapsed / self.duration).clamp(0.0, 1.0)
        };
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

    fn advance_direction(&mut self) {
        match self.direction {
            AnimationDirection::Normal => {
                self.forward = true;
            }
            AnimationDirection::Reverse => {
                self.forward = false;
            }
            AnimationDirection::Alternate => {
                self.forward = !self.forward;
            }
            AnimationDirection::AlternateReverse => {
                self.forward = !self.forward;
            }
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_finished(&self) -> bool {
        !self.repeat && self.elapsed >= self.duration
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

    fn clone_box(&self) -> Box<dyn crate::Animate> {
        Box::new(self.clone())
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
