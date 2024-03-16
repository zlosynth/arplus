pub struct Continuous {
    value: f32,
}

impl Continuous {
    pub fn new() -> Self {
        Self { value: 0.0 }
    }

    pub fn reconcile(&mut self, value: f32) {
        self.value = value;
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

pub fn linear_sum(pot: f32, cv: Option<f32>) -> f32 {
    let offset_cv = cv.unwrap_or(0.0) / 5.0;
    let sum = pot + offset_cv;
    let clamped = sum.clamp(0.0, 1.0);
    clamped
}
