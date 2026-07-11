use crate::AnimationDirection;

#[derive(Clone)]
pub struct Spring {
    start: f32,
    value: f32,
    velocity: f32,
    target: f32,
    mass: f32,
    stiffness: f32,
    damping: f32,
    threshold: f32,
    repeat: bool,
    direction: AnimationDirection,
}

impl Spring {
    pub fn new(value: f32) -> Self {
        Self {
            start: value,
            value,
            velocity: 0.0,
            target: value,
            mass: 0.75,
            stiffness: 80.0,
            damping: 10.0,
            threshold: 0.01,
            repeat: false,
            direction: AnimationDirection::Normal,
        }
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

    pub fn repeat(mut self) -> Self {
        self.repeat = true;
        self
    }

    pub fn direction(mut self, direction: AnimationDirection) -> Self {
        self.direction = direction;
        self
    }

    pub fn snap_to(&mut self, target: f32) {
        self.target = target;
    }

    pub fn tick(&mut self, dt: f32) -> f32 {
        let dt = dt.min(0.05);
        let diff = self.target - self.value;
        let acceleration = (self.stiffness * diff - self.damping * self.velocity) / self.mass;
        self.velocity += acceleration * dt;
        self.value += self.velocity * dt;

        if self.is_at_rest() {
            if self.repeat {
                self.value = self.target;
                self.velocity = 0.0;
                self.advance_direction();
            } else {
                self.value = self.target;
                self.velocity = 0.0;
            }
        }

        self.value
    }

    fn advance_direction(&mut self) {
        match self.direction {
            AnimationDirection::Normal => {
                self.value = self.start;
            }
            AnimationDirection::Reverse => {
                self.value = self.start;
            }
            AnimationDirection::Alternate => {
                std::mem::swap(&mut self.start, &mut self.target);
            }
            AnimationDirection::AlternateReverse => {
                std::mem::swap(&mut self.start, &mut self.target);
            }
        }
        self.velocity = 0.0;
    }

    pub fn value(&self) -> f32 {
        self.value
    }

    pub fn is_at_rest(&self) -> bool {
        (self.target - self.value).abs() < self.threshold
            && self.velocity.abs() < self.threshold
    }
}

impl crate::Animate for Spring {
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
