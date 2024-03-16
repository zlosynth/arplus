use super::primitives::continuous::Continuous;
use super::primitives::math;

pub struct Resonance {
    continuous: Continuous,
}

impl Resonance {
    pub fn new() -> Self {
        Self {
            continuous: Continuous::new(),
        }
    }

    pub fn reconcile(&mut self, pot: f32, cv: Option<f32>) {
        self.continuous.reconcile(math::linear_sum(pot, cv));
    }

    pub fn value(&self) -> f32 {
        self.continuous.value()
    }
}
