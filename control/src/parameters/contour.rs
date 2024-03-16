use super::primitives::continuous::Continuous;

pub struct Contour {
    continuous: Continuous,
}

impl Contour {
    pub fn new() -> Self {
        Self {
            continuous: Continuous::new(),
        }
    }

    pub fn reconcile(&mut self, pot: f32, cv: Option<f32>) {
        self.continuous.reconcile(linear_sum(pot, cv));
    }

    pub fn value(&self) -> f32 {
        self.continuous.value()
    }
}

// TODO: Move it to a lib?
fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
