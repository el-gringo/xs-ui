use std::time::Duration;

use crate::AnimationDirection;

#[derive(Clone, Copy, PartialEq)]
enum EasingTiming {
    Ease,
    EaseIn,
    EaseInOut,
    EaseOut,
    Linear,
}

#[derive(Clone, PartialEq)]
pub struct Easing {
    value: f32,
    duration: Duration,
    elapsed: Duration,
    timing: EasingTiming,
    repeat: u32,
    direction: AnimationDirection,
    forward: bool,
}

impl Default for Easing {
    fn default() -> Self {
        Self {
            value: 0.0,
            duration: Duration::from_millis(150),
            elapsed: Duration::ZERO,
            timing: EasingTiming::Linear,
            repeat: 0,
            direction: AnimationDirection::Normal,
            forward: true,
        }
    }
}

impl Easing {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn repeat(mut self, times: u32) -> Self {
        self.repeat = times;
        self
    }

    pub fn direction(mut self, direction: AnimationDirection) -> Self {
        self.direction = direction;
        match direction {
            AnimationDirection::Normal | AnimationDirection::Alternate => self.forward = true,
            AnimationDirection::Reverse | AnimationDirection::AlternateReverse => {
                self.forward = false;
            }
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

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    fn progress(&self) -> f32 {
        let t = (self.elapsed.as_secs_f32() / self.duration.as_secs_f32()).clamp(0.0, 1.0);
        if self.forward { t } else { 1.0 - t }
    }

    fn eased(&self) -> f32 {
        let t = self.progress();
        match self.timing {
            EasingTiming::Linear => t,
            EasingTiming::EaseIn => ease_in(t),
            EasingTiming::EaseOut => ease_out(t),
            EasingTiming::EaseInOut => ease_in_out(t),
            EasingTiming::Ease => ease_in_out(t),
        }
    }

    pub fn tick(&mut self, dt: f32) -> f32 {
        self.elapsed += Duration::from_secs_f32(dt);
        while self.repeat > 0 && self.elapsed >= self.duration {
            self.elapsed = self.elapsed.saturating_sub(self.duration);
            self.repeat -= 1;
            self.advance_direction();
        }
        self.value = self.eased();
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
            AnimationDirection::Alternate | AnimationDirection::AlternateReverse => {
                self.forward = !self.forward;
            }
        }
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_finished(&self) -> bool {
        self.repeat == 0 && self.elapsed >= self.duration
    }
}

impl crate::Animate for Easing {
    fn reset(&mut self) {
        self.elapsed = Duration::ZERO;
        self.value = 0.;
        self.forward = !matches!(self.direction, AnimationDirection::Reverse | AnimationDirection::AlternateReverse);
    }

    fn tick(&mut self, dt: f32) -> f32 {
        self.tick(dt)
    }

    fn value(&self) -> f32 {
        self.value()
    }

    fn is_finished(&self) -> bool {
        self.repeat == 0 && self.elapsed >= self.duration
    }

    fn clone_box(&self) -> Box<dyn crate::Animate> {
        Box::new(self.clone())
    }

    fn same_config(&self, other: &dyn crate::Animate) -> bool {
        if std::any::Any::type_id(self) != std::any::Any::type_id(other) {
            return false;
        }
        let other = (other as &dyn std::any::Any)
            .downcast_ref::<Self>()
            .unwrap();
        self == other
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::AnimationDirection::Alternate;

    #[test]
    fn animates_from_zero_to_one_over_duration() {
        let mut easing = Easing::new().duration(Duration::from_millis(1000));
        assert_eq!(easing.value(), 0.0);
        assert!(!easing.is_finished());

        let mut max = 0.0f32;
        for _ in 0..120 {
            let v = easing.tick(1.0 / 60.0);
            max = max.max(v);
        }
        assert!(
            (max - 1.0).abs() < 0.001,
            "peak should reach 1.0, got {max}"
        );
        assert!(easing.is_finished());
        assert_eq!(easing.value(), 1.0);
    }

    #[test]
    fn alternate_bounces_between_zero_and_one() {
        let mut easing = Easing::new()
            .duration(Duration::from_millis(1000))
            .repeat(u32::MAX)
            .direction(Alternate);
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for _ in 0..360 {
            let v = easing.tick(1.0 / 60.0);
            min = min.min(v);
            max = max.max(v);
        }
        assert!((max - 1.0).abs() < 0.01, "peak should reach 1.0, got {max}");
        assert!(min < 0.01, "should bounce back to zero, got {min}");
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
