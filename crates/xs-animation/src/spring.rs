use crate::AnimationDirection;

#[derive(Clone, PartialEq)]
pub struct Spring {
    value: f32,
    velocity: f32,
    forward: bool,
    mass: f32,
    stiffness: f32,
    damping: f32,
    threshold: f32,
    repeat: u32,
    direction: AnimationDirection,
}

impl Default for Spring {
    fn default() -> Self {
        Self {
            value: 0.0,
            velocity: 0.0,
            forward: true,
            mass: 1.,
            stiffness: 80.,
            damping: 10.,
            threshold: 0.001,
            repeat: 0,
            direction: AnimationDirection::Normal,
        }
    }
}

impl Spring {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    pub fn stiffness(mut self, stiffness: f32) -> Self {
        self.stiffness = stiffness;
        self
    }

    pub fn damping(mut self, damping: f32) -> Self {
        self.damping = damping;
        self
    }

    pub fn repeat(mut self, times: u32) -> Self {
        self.repeat = times;
        self
    }

    pub fn direction(mut self, direction: AnimationDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn tick(&mut self, dt: f32) -> f32 {
        let dt = dt.min(0.05);
        let goal = if self.forward { 1.0 } else { 0.0 };
        let diff = goal - self.value;
        let acceleration = (self.stiffness * diff - self.damping * self.velocity) / self.mass;
        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;

        if self.is_at_rest() {
            self.value = goal;
            self.velocity = 0.0;
            if self.repeat > 0 {
                self.repeat -= 1;
                self.advance_direction();
            }
        }
        self.value
    }

    fn advance_direction(&mut self) {
        match self.direction {
            AnimationDirection::Normal | AnimationDirection::Reverse => {
                self.value = 0.0;
                self.forward = true;
            }
            AnimationDirection::Alternate | AnimationDirection::AlternateReverse => {
                self.forward = !self.forward;
            }
        }
        self.velocity = 0.0;
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_at_rest(&self) -> bool {
        let goal = if self.forward { 1.0 } else { 0.0 };
        let threshold = self.threshold;
        (goal - self.value).abs() < threshold && self.velocity.abs() < threshold
    }
}

impl crate::Animate for Spring {
    fn reset(&mut self) {
        self.value = 0.;
        self.velocity = 0.;
        self.forward = true;
    }
    fn tick(&mut self, dt: f32) -> f32 {
        self.tick(dt)
    }

    fn value(&self) -> f32 {
        self.value()
    }

    fn is_finished(&self) -> bool {
        self.repeat == 0 && self.is_at_rest()
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
    use super::*;
    use crate::Animate;
    use crate::AnimationDirection::Alternate;

    #[test]
    fn starts_from_zero_and_animates_toward_target() {
        let mut spring = Spring::new();
        assert_eq!(spring.value(), 0.0);
        assert!(!spring.is_finished());

        let mut max = 0.0f32;
        for _ in 0..2000 {
            let v = spring.tick(1.0 / 60.0);
            max = max.max(v);
        }
        assert!(
            (max - 1.0).abs() < 0.15,
            "peak should approach target, got {max}"
        );
        assert!(spring.is_finished());
    }

    #[test]
    fn alternate_oscillates_back_and_forth() {
        let mut spring = Spring::new().repeat(u32::MAX).direction(Alternate);
        let mut min = f32::MAX;
        let mut max = f32::MIN;
        for _ in 0..2000 {
            let v = spring.tick(1.0 / 60.0);
            min = min.min(v);
            max = max.max(v);
        }
        assert!(
            (max - 1.0).abs() < 0.15,
            "peak should reach target, got {max}"
        );
        assert!(min < 0.05, "should swing back to zero, got {min}");
    }
}
