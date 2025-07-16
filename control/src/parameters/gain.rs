use super::primitives::continuous::Continuous;
use super::primitives::math;

pub struct Gain {
    continuous: Continuous,
}

impl Gain {
    pub fn new() -> Self {
        Self {
            continuous: Continuous::new(),
        }
    }

    pub fn reconcile(&mut self, pot: f32, cv: Option<f32>) {
        self.continuous.reconcile(math::linear_sum(pot, cv));
    }

    pub fn value(&self) -> f32 {
        self.continuous.value() * 2.0
    }
}
